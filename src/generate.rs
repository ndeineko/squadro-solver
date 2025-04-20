use std::io::{self, Write};

use roaring::RoaringTreemap;

use crate::board_state::BoardState;
use crate::file_operations;

/// Generate data files needed to play a game
///
/// Generate one data file with winning states per player and one file with all explored states.
pub fn generate(init_states: &[BoardState]) {
    // Make sure the data files do not already exist.
    check_before_generate();

    println!("Generating states. This will take a while.");

    let mut remaining_states = collect_reachable_states(init_states);

    // Save all states seen during exploration.
    file_operations::write_states(file_operations::ALL_STATES_PATH, &remaining_states);
    println!("{} explored states saved.", remaining_states.len());

    let player_0_winning_states = collect_winning_states(&mut remaining_states);

    // Save winning states for player 0.
    file_operations::write_states(
        file_operations::WINNING_STATES_PATH[0],
        &player_0_winning_states,
    );
    println!(
        "{} winning states saved for player 0.",
        player_0_winning_states.len()
    );

    remaining_states |= player_0_winning_states;
    let player_1_winning_states = collect_reachable_states(init_states) - remaining_states;

    // Save winning states for player 1.
    file_operations::write_states(
        file_operations::WINNING_STATES_PATH[1],
        &player_1_winning_states,
    );
    println!(
        "{} winning states saved for player 1.",
        player_1_winning_states.len()
    );
}

/// Return all states reachable from at least one of the `init_states`
fn collect_reachable_states(init_states: &[BoardState]) -> RoaringTreemap {
    let mut reachable_states = RoaringTreemap::new();

    for state in init_states {
        // Mark all explored states.
        collect_reachable_states_recursively(state.clone(), &mut reachable_states);
    }

    reachable_states
}

/// Recursively (depth-first order) mark states reachable from `current_state`
#[decurse::decurse_unsound]
fn collect_reachable_states_recursively(
    current_state: BoardState,
    reachable_states: &mut RoaringTreemap,
) {
    // Note: `insert` returns `false` if `current_state.get_id()` is already in `reachable_states`.
    if !reachable_states.insert(current_state.get_id()) || current_state.is_ended() {
        return;
    }

    for next_state in current_state.get_next_states() {
        // Explore recursively.
        collect_reachable_states_recursively(next_state, reachable_states);
    }
}

/// Return all winning states of player 0
///
/// Initially, `remaining_states` must contain all reachable states.
/// After calling this function, `remaining_states` will contain the states for which neither player can guarantee a win.
fn collect_winning_states(remaining_states: &mut RoaringTreemap) -> RoaringTreemap {
    let mut player_0_winning_states = RoaringTreemap::new();

    let mut previous_remaining_states_len: u64 = remaining_states.len();
    let mut previous_player_0_winning_states_len: u64 = player_0_winning_states.len();

    // Explore `remaining_states` several times until no new winning state can be found.
    for iteration in 1.. {
        print!("Iteration {} ... ", iteration);
        // Without flushing, nothing is printed until the next newline.
        io::stdout().flush().expect("stdout should be writable");

        collect_winning_states_scan_remaining(remaining_states, &mut player_0_winning_states);

        let remaining_states_diff = previous_remaining_states_len - remaining_states.len();
        let player_0_winning_states_diff =
            player_0_winning_states.len() - previous_player_0_winning_states_len;

        println!(
            "Found {} new winning states for player 0 and {} for player 1.",
            player_0_winning_states_diff,
            remaining_states_diff - player_0_winning_states_diff
        );

        if remaining_states_diff == 0 {
            break;
        }

        previous_remaining_states_len = remaining_states.len();
        previous_player_0_winning_states_len = player_0_winning_states.len();
    }

    player_0_winning_states
}

/// Scan `remaining_states` linearly to find new winning states and mark winning states of player 0
///
/// Since loops can occur in a game, this must be called multiple times until `remaining_states` stops shrinking.
fn collect_winning_states_scan_remaining(
    remaining_states: &mut RoaringTreemap,
    player_0_winning_states: &mut RoaringTreemap,
) {
    // From here until the clean up, if a state ID is in `remaining_states` AND in `seen_or_player_0_winning_states`,
    // then the corresponding state has been seen but was not found winning in the current iteration.
    let seen_or_player_0_winning_states = player_0_winning_states;

    let mut next_state_id_from = 0;
    while let Some(state_id) = treemap_next_value(remaining_states, next_state_id_from) {
        collect_winning_states_recursively(
            BoardState::from(state_id),
            remaining_states,
            seen_or_player_0_winning_states,
        );
        next_state_id_from = state_id + 1;
    }

    // Clean up `seen_or_player_0_winning_states` to only keep IDs of winning states.
    for state_id in remaining_states.iter() {
        seen_or_player_0_winning_states.remove(state_id);
    }
}

/// From `current_state`, scan `remaining_states` recursively (depth-first order) to find new winning states and mark winning states of player 0
///
/// The return value corresponds to the winning player of `current_state`. The value is -1 for a draw (or when the winner is currently unknown).
/// Since loops can occur in a game, some winning states will only be found after calling this function multiple times for the same `current_state`.
#[decurse::decurse_unsound]
fn collect_winning_states_recursively(
    current_state: BoardState,
    remaining_states: &mut RoaringTreemap,
    seen_or_player_0_winning_states: &mut RoaringTreemap,
) -> isize {
    let current_state_id = current_state.get_id();

    // If `current_state_id` is not in `remaining_states`, then `current_state` is winning for one of the players.
    if !remaining_states.contains(current_state_id) {
        // Return the winning player.
        return !seen_or_player_0_winning_states.contains(current_state_id) as isize;
    }

    // Note: `insert` returns `false` if `current_state_id` is already in `seen_or_player_0_winning_states`.
    if !seen_or_player_0_winning_states.insert(current_state_id) {
        // Inconsistencies may arise if `current_state_id` is also an ancestor state.
        // In that case, we may not yet know if `current_state_id` is winning or not,
        // which is why the current function must be called multiple times.
        return -1; // `current_state` has been seen but was not found winning (it could be a draw or currently unknown win).
    }

    if current_state.is_ended() {
        remaining_states.remove(current_state_id);
        if current_state.get_next_player() == 0 {
            seen_or_player_0_winning_states.remove(current_state_id);
            return 1; // Game ends with a win for player 1.
        }
        return 0; // Game ends with a win for player 0.
    }

    let next_player = current_state.get_next_player() as isize;
    let last_player = 1 - next_player;

    // `current_eval` starts with the worst case for `next_player` (a loss).
    let mut current_eval = last_player;

    for next_state in current_state.get_next_states() {
        // Explore recursively.
        #[rustfmt::skip]
        let next_state_eval = collect_winning_states_recursively(
            next_state,
            remaining_states,
            seen_or_player_0_winning_states
        );

        if next_state_eval == -1 {
            // If one of the next states is a draw (or currently unknown win), the worst case is a draw.
            current_eval = -1;
        } else if next_state_eval == next_player {
            // Once a next state is winning for `next_player`, then `current_state` is winning for `next_player`.

            // Update the bit-sets to define `current_state` as winning for `next_player`.
            remaining_states.remove(current_state_id);
            if next_player != 0 {
                seen_or_player_0_winning_states.remove(current_state_id);
            }

            return next_player;
        }
    }

    if current_eval == last_player {
        // Update the bit-sets to define `current_state` as loosing for `next_player`.
        remaining_states.remove(current_state_id);
        if next_player == 0 {
            seen_or_player_0_winning_states.remove(current_state_id);
        }
    }

    current_eval
}

/// Terminate thread if `generate` would write to a file that already exists
fn check_before_generate() {
    file_operations::abort_if_path_exists(file_operations::ALL_STATES_PATH);

    for player in 0..=1 {
        file_operations::abort_if_path_exists(file_operations::WINNING_STATES_PATH[player]);
    }
}

/// Get the next value in `treemap`, starting from (and including) `from`
///
/// Return `None` when there is no next value.
fn treemap_next_value(treemap: &RoaringTreemap, from: u64) -> Option<u64> {
    let from_high = (from >> 32) as u32;
    let from_low = from as u32;

    treemap
        .bitmaps()
        .skip_while(|(high, _container)| *high < from_high)
        .flat_map(|(high, container)| {
            container
                .range(if high > from_high { 0.. } else { from_low.. })
                .next()
                .map(|low| ((high as u64) << 32) | (low as u64))
        })
        .next()
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[test]
    fn data_generation() {
        let init_state = BoardState::from(85065666045);

        let get_generate_result = || {
            std::panic::catch_unwind(|| {
                generate(&[init_state.clone()]);
            })
        };

        let get_state_value = |player_opt, id| {
            let path = match player_opt {
                None => file_operations::ALL_STATES_PATH,
                Some(player) => file_operations::WINNING_STATES_PATH[player],
            };

            file_operations::read_state_value(path, id)
        };

        file_operations::tests::run_in_tempdir(|| {
            assert!(get_generate_result().is_ok());

            let result = get_generate_result();
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .downcast::<String>()
                .unwrap()
                .contains("already exists"));

            for piece in [0, 1, 4] {
                assert!(get_state_value(
                    None,
                    init_state.get_next_state(piece).unwrap().get_id()
                ));
            }

            for player in 0..=1 {
                assert_eq!(
                    get_state_value(Some(player), init_state.get_id()),
                    player == 1
                );
                assert_eq!(
                    get_state_value(Some(player), init_state.get_next_state(0).unwrap().get_id()),
                    player == 0
                );
                assert_eq!(
                    get_state_value(Some(player), init_state.get_next_state(1).unwrap().get_id()),
                    player == 0
                );
                assert_eq!(
                    get_state_value(Some(player), init_state.get_next_state(4).unwrap().get_id()),
                    player == 1
                );
            }

            for _i in 0..25 {
                let mut state = init_state.clone();
                while !state.is_ended() {
                    let next_player = state.get_next_player();
                    let next_states: Vec<BoardState> = state.get_next_states().collect();
                    let next_non_loosing_states: Vec<BoardState> = state
                        .get_next_states()
                        .filter(|s| get_state_value(Some(next_player), s.get_id()))
                        .collect();

                    if next_player == 0 {
                        assert!(next_non_loosing_states.is_empty());
                        assert!(!next_states.is_empty());
                        state = next_states[fastrand::usize(0..next_states.len())].clone();
                    } else {
                        assert!(!next_non_loosing_states.is_empty());
                        state = next_non_loosing_states
                            [fastrand::usize(0..next_non_loosing_states.len())]
                        .clone();
                    }
                }

                assert!(state.get_next_player() == 0);
            }
        });
    }

    #[test]
    fn player_data_generation() {
        let init_state = BoardState::from(5057791486);

        let get_generate_result = || {
            std::panic::catch_unwind(|| {
                generate(&[init_state.clone()]);
            })
        };

        let get_state_value = |player_opt, id| {
            let path = match player_opt {
                None => file_operations::ALL_STATES_PATH,
                Some(player) => file_operations::WINNING_STATES_PATH[player],
            };

            file_operations::read_state_value(path, id)
        };

        file_operations::tests::run_in_tempdir(|| {
            assert!(get_generate_result().is_ok());

            assert!(get_state_value(None, init_state.get_id()));
            for player in 0..=1 {
                assert!(!get_state_value(
                    None,
                    BoardState::new_game(player).get_id()
                ));
                assert!(!get_state_value(Some(player), init_state.get_id()));

                assert_eq!(
                    get_state_value(Some(player), init_state.get_next_state(0).unwrap().get_id()),
                    player == 1
                );
                assert_eq!(
                    get_state_value(
                        Some(player),
                        init_state
                            .get_next_state(0)
                            .unwrap()
                            .get_next_state(0)
                            .unwrap()
                            .get_id()
                    ),
                    player == 0
                );
                assert_eq!(
                    get_state_value(Some(player), init_state.get_next_state(2).unwrap().get_id()),
                    player == 1
                );
                assert_eq!(
                    get_state_value(
                        Some(player),
                        init_state
                            .get_next_state(2)
                            .unwrap()
                            .get_next_state(0)
                            .unwrap()
                            .get_id()
                    ),
                    player == 0
                );
                assert_eq!(
                    get_state_value(
                        Some(player),
                        init_state
                            .get_next_state(3)
                            .unwrap()
                            .get_next_state(3)
                            .unwrap()
                            .get_id()
                    ),
                    player == 0
                );
            }

            let mut state = init_state.clone();
            let mut loop_count = 0;
            while loop_count < 25 {
                for s in state.get_next_states() {
                    assert!(get_state_value(None, s.get_id()));
                }

                let next_non_loosing_states: Vec<BoardState> = state
                    .get_next_states()
                    .filter(|s| !get_state_value(Some(s.get_next_player()), s.get_id()))
                    .collect();

                assert!(!next_non_loosing_states.is_empty());

                for s in &next_non_loosing_states {
                    assert!(!s.is_ended());
                    assert!(!get_state_value(Some(state.get_next_player()), s.get_id()));
                }

                if state.get_id() == init_state.get_id() {
                    assert_eq!(next_non_loosing_states.len(), 1);
                    assert_eq!(next_non_loosing_states[0].get_id(), 5057794943);
                    loop_count += 1;
                }

                state = next_non_loosing_states[fastrand::usize(0..next_non_loosing_states.len())]
                    .clone();
            }
        });
    }

    #[test]
    fn simple_endgame_exploration() {
        let init_state = BoardState::from(100382226046);

        let seen_states = collect_reachable_states(&[init_state.clone()]);

        let mut remaining_states = seen_states.clone();
        let mut winning_states = collect_winning_states(&mut remaining_states);

        let init_state_is_winning = winning_states.contains(init_state.get_id());

        assert!(init_state_is_winning);
        assert_eq!(seen_states.len(), 3);
        assert_eq!(seen_states, winning_states);
        assert!(seen_states.contains(100382226046));
        assert!(seen_states.contains(100382226046 + 60217344 + 1));
        assert!(seen_states.contains(100382226046 + 3456 + 1));

        winning_states = &seen_states - (remaining_states | winning_states);

        let init_state_is_winning = winning_states.contains(init_state.get_id());

        assert!(!init_state_is_winning);
        assert_eq!(winning_states.len(), 0);
        assert_eq!(seen_states.intersection_len(&winning_states), 0);
    }

    #[test]
    fn tricky_endgame_exploration() {
        let init_state = BoardState::from(85065666045);

        let mut previous_seen_states_len = 0;

        for player in 0..=1 {
            let seen_states = collect_reachable_states(&[init_state.clone()]);

            let mut remaining_states = seen_states.clone();
            let mut winning_states = collect_winning_states(&mut remaining_states);

            if player == 1 {
                winning_states = &seen_states - (remaining_states | winning_states);
            }

            let init_state_is_winning = winning_states.contains(init_state.get_id());
            assert_eq!(init_state_is_winning, player == 1);

            assert_eq!(previous_seen_states_len == seen_states.len(), player == 1);
            previous_seen_states_len = seen_states.len();

            assert_eq!(winning_states.contains(init_state.get_id()), player == 1);
            assert_eq!(
                winning_states.contains(init_state.get_next_state(0).unwrap().get_id()),
                player == 0
            );
            assert_eq!(
                winning_states.contains(init_state.get_next_state(1).unwrap().get_id()),
                player == 0
            );
            assert_eq!(
                winning_states.contains(init_state.get_next_state(4).unwrap().get_id()),
                player == 1
            );
        }
    }

    #[test]
    fn endless_game_exploration() {
        let init_state = BoardState::from(5057791486);

        let mut seen_states_vec: Vec<RoaringTreemap> = Vec::new();
        let mut winning_states_vec: Vec<RoaringTreemap> = Vec::new();

        for player in 0..=1 {
            let seen_states = collect_reachable_states(&[init_state.clone()]);

            let mut remaining_states = seen_states.clone();
            let mut winning_states = collect_winning_states(&mut remaining_states);

            if player == 1 {
                winning_states = &seen_states - (remaining_states | winning_states);
            }

            let init_state_is_winning = winning_states.contains(init_state.get_id());
            assert!(!init_state_is_winning);

            assert!(!winning_states.is_empty());
            assert!(seen_states.len() > winning_states.len());

            seen_states_vec.push(seen_states);
            winning_states_vec.push(winning_states);
        }

        assert_eq!(seen_states_vec[0], seen_states_vec[1]);
        assert_eq!(
            winning_states_vec[0].intersection_len(&winning_states_vec[1]),
            0
        );
        assert!(
            seen_states_vec[0].len() > winning_states_vec[0].len() + winning_states_vec[1].len()
        );

        let mut state = init_state.clone();
        let mut loop_count = 0;
        while loop_count < 25 {
            let next_non_loosing_states: Vec<BoardState> = state
                .get_next_states()
                .filter(|s| !winning_states_vec[1 - state.get_next_player()].contains(s.get_id()))
                .collect();

            for s in &next_non_loosing_states {
                assert!(!s.is_ended());
                for winning_states in &winning_states_vec {
                    assert!(!winning_states.contains(s.get_id()));
                }
            }

            if state.get_id() == init_state.get_id() {
                assert_eq!(next_non_loosing_states.len(), 1);
                assert_eq!(next_non_loosing_states[0].get_id(), 5057794943);
                loop_count += 1;
            }

            state =
                next_non_loosing_states[fastrand::usize(0..next_non_loosing_states.len())].clone();
        }
    }

    #[test]
    fn mistake_protection() {
        let get_check_result = || {
            std::panic::catch_unwind(|| {
                check_before_generate();
            })
        };

        for path in [file_operations::ALL_STATES_PATH]
            .iter()
            .chain(file_operations::WINNING_STATES_PATH.iter())
        {
            file_operations::tests::run_in_tempdir(|| {
                assert!(get_check_result().is_ok());

                File::create(path).unwrap();

                let result = get_check_result();
                assert!(result.is_err());
                assert!(result
                    .unwrap_err()
                    .downcast::<String>()
                    .unwrap()
                    .contains(path));
            });
        }
    }
}
