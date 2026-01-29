use std::io::{self, BufRead, Write};

use crate::board_state::BoardState;
use crate::file_operations;

/// Evaluation of the board state
#[derive(Debug, PartialEq)]
enum BoardStateEval {
    Win,
    Draw, // Endless game.
    Loss,
}

/// Play a game, starting from the board state represented by `init_id`
///
/// Return all states encountered during the game and the winner of the game.
pub fn play(
    init_id: u64,
    human_player_opt: Option<usize>,
    show_eval: bool,
) -> (Vec<BoardState>, usize) {
    abort_if_id_is_invalid(init_id);

    let init_state = BoardState::from(init_id);
    match human_player_opt {
        Some(human_player) => {
            // Start playing against computer.
            let (all_states, winner) = print_all_states(
                init_state,
                &|state: BoardState| -> (Option<BoardState>, Option<BoardStateEval>) {
                    if state.get_next_player() == human_player {
                        get_next_state_from_user_input(state, io::stdin().lock())
                    } else {
                        get_best_next_state(state)
                    }
                },
                show_eval,
            );

            if winner == human_player {
                println!("\nHuman wins!");
            } else {
                println!("\nComputer wins!");
            }

            (all_states, winner)
        }
        None => {
            // Start computer self-play.
            print_all_states(init_state, &get_best_next_state, show_eval)
        }
    }
}

/// Starting from `init_state`, print states provided by `get_next_state` and stop when the game ends
///
/// Return all printed states and the winner of the game.
fn print_all_states(
    init_state: BoardState,
    get_next_state: &dyn Fn(BoardState) -> (Option<BoardState>, Option<BoardStateEval>),
    show_eval: bool,
) -> (Vec<BoardState>, usize) {
    let mut state = init_state;
    let mut all_states = vec![state.clone()];

    println!("{}", state);

    while !state.is_ended() {
        let (state_opt, eval_opt) = get_next_state(state.clone());
        if state_opt.is_none() {
            println!("\n(Player resigned)");
            break;
        }
        state = state_opt.expect("The state should exist");

        all_states.push(state.clone());

        println!("\n{}", state);

        if let (true, Some(eval)) = (show_eval, eval_opt) {
            println!("(Last player's evaluation : {:?})", eval);
        }
    }

    (all_states, 1 - state.get_next_player())
}

/// Ask the user for their next move and return the corresponding next state
fn get_next_state_from_user_input(
    state: BoardState,
    mut reader: impl BufRead,
) -> (Option<BoardState>, Option<BoardStateEval>) {
    loop {
        print!("\nYour move : "); // Without flushing, that string is printed after user input.
        io::stdout().flush().expect("stdout should be writable");

        // Read user input from stdin.
        let mut input = String::new();
        match reader.read_line(&mut input) {
            Ok(0) => return (None, None), // End of user input.
            Ok(_) => {
                if let Ok(input_usize) = input.trim().parse::<usize>() {
                    if let Some(next_state) = state.get_next_state(input_usize) {
                        // If the user-given piece is valid, return the corresponding state.
                        return (Some(next_state), None);
                    }
                }
            }
            Err(e) => match e.kind() {
                io::ErrorKind::InvalidData => {} // Invalid UTF-8 byte sequence.
                _ => eprintln!("Error : {}", e),
            },
        };

        let available_pieces = (0..5)
            .filter_map(|p| state.get_next_state(p).map(|_| p.to_string()))
            .collect::<Vec<String>>()
            .join(", ");
        print!("Invalid move! Available piece(s) : {}", available_pieces);
    }
}

/// Return a next state that gives the best final outcome for the next player
fn get_best_next_state(state: BoardState) -> (Option<BoardState>, Option<BoardStateEval>) {
    let next_player = state.get_next_player();

    let mut next_states: Vec<BoardState> = state.get_next_states().collect();
    fastrand::shuffle(&mut next_states);

    // Look for a winning state in `next_states`.
    for next_state in &next_states {
        if file_operations::read_state_value(
            file_operations::WINNING_STATES_PATH[next_player],
            next_state.get_id(),
        ) {
            // Return a winning state.
            return (Some(next_state.clone()), Some(BoardStateEval::Win));
        }
    }

    // Look for a non-winning state (for the previous player) in `next_states`.
    for next_state in &next_states {
        if !file_operations::read_state_value(
            file_operations::WINNING_STATES_PATH[1 - next_player],
            next_state.get_id(),
        ) {
            // Return a drawing state.
            return (Some(next_state.clone()), Some(BoardStateEval::Draw));
        }
    }

    // Return a losing state.
    (
        Some(
            next_states
                .first()
                .expect("There should be at least one next state")
                .clone(),
        ),
        Some(BoardStateEval::Loss),
    )
}

/// Terminate thread if `id` does not represent a valid board state
fn abort_if_id_is_invalid(id: u64) {
    if !file_operations::read_state_value(file_operations::ALL_STATES_PATH, id) {
        panic!("Invalid board state ID : {}", id);
    }
}

#[cfg(test)]
mod tests {
    use std::slice;

    use crate::generate::generate;

    use super::*;

    #[test]
    fn validate_id_and_play() {
        let get_play_result =
            |id, human_player_opt| std::panic::catch_unwind(|| play(id, human_player_opt, false));

        let init_state = BoardState::from(100382226046);

        let err_id = [0, 1, 5057791486, 85065666045];
        let ok_id = [init_state.get_id(), 100382229503, 100442443391];

        file_operations::tests::run_in_tempdir(|| {
            for &id in err_id.iter().chain(ok_id.iter()) {
                assert!(get_play_result(id, None).is_err());
            }

            generate(slice::from_ref(&init_state));

            for id in err_id {
                assert!(get_play_result(id, None).is_err());
            }

            for id in ok_id {
                assert!(get_play_result(id, None).is_ok());
            }
        });
    }

    #[test]
    fn computer_self_play() {
        let init_state = BoardState::from(85065666045);

        file_operations::tests::run_in_tempdir(|| {
            generate(slice::from_ref(&init_state));

            for _i in 0..25 {
                let first_moved_piece = vec![0, 1, 4][fastrand::usize(0..3)];
                let second_state = init_state
                    .get_next_state(first_moved_piece)
                    .expect("Pieces 0, 1 and 4 should be movable");

                let (all_states, winner) = play(second_state.get_id(), None, false);

                assert_eq!(winner, if first_moved_piece == 4 { 1 } else { 0 });
                assert_eq!(winner, all_states.len() % 2);

                assert!(!all_states.is_empty());
                assert!(all_states.last().unwrap().is_ended());

                for (index, state) in all_states.iter().enumerate() {
                    assert_eq!(state.get_next_player(), index % 2);

                    if index == 0 {
                        assert_eq!(state.get_id(), second_state.get_id());
                    } else {
                        assert!(all_states[index - 1]
                            .get_next_states()
                            .any(|s| s.get_id() == state.get_id()));
                    }
                }
            }
        });
    }

    #[test]
    fn play_and_await_input() {
        use std::sync::mpsc;

        let init_id = 100382226046;
        let init_state = BoardState::from(init_id);

        file_operations::tests::run_in_tempdir(|| {
            generate(slice::from_ref(&init_state));

            for human_player in (0..=1).rev() {
                let (send, recv) = mpsc::channel();

                let thread_handle = std::thread::spawn(move || {
                    // The following call should never end IFF `human_player` is 0 AND stdin exists.
                    let (all_states, winner) = play(init_id, Some(human_player), false);

                    assert_eq!(winner, 1 - human_player);
                    assert_eq!(all_states.len(), 1 + human_player);

                    let last_state = all_states.last().unwrap();
                    assert_eq!(last_state.is_ended(), human_player == 1);
                    assert_eq!(last_state.get_next_player(), human_player);

                    send.send(true).unwrap();
                });

                match recv.recv_timeout(std::time::Duration::from_millis(5000)) {
                    Err(mpsc::RecvTimeoutError::Timeout) => assert_eq!(human_player, 0),
                    _ => thread_handle.join().unwrap(), // Propagate possible panic in subthread.
                }
            }
        });
    }

    #[test]
    fn print_all_and_win() {
        for _i in 0..25 {
            let mut state = BoardState::from(85065666045);
            let mut random_next_states = vec![state.clone()];
            while !state.is_ended() {
                let next_states: Vec<BoardState> = state.get_next_states().collect();
                state = next_states[fastrand::usize(0..next_states.len())].clone();
                random_next_states.push(state.clone());
            }
            assert!(random_next_states.len() >= 4);

            let get_next_state = |state: BoardState| {
                let current_index_opt = random_next_states
                    .iter()
                    .position(|s| s.get_id() == state.get_id());
                assert!(current_index_opt.is_some());

                let next_index = current_index_opt.unwrap() + 1;
                if next_index == random_next_states.len() {
                    (None, None)
                } else {
                    (Some(random_next_states[next_index].clone()), None)
                }
            };

            let (all_states, winner) =
                print_all_states(random_next_states[0].clone(), &get_next_state, false);

            assert_eq!(all_states.len(), random_next_states.len());
            for (index, state) in all_states.iter().enumerate() {
                assert_eq!(state.get_id(), random_next_states[index].get_id());
            }

            assert_eq!(1 - winner, all_states.len() % 2);
        }
    }

    #[test]
    fn print_all_and_resign() {
        let mut next_states = vec![BoardState::new_game(1)];
        for piece in 0..5 {
            for _player in (0..=1).rev() {
                next_states.push(next_states.last().unwrap().get_next_state(piece).unwrap());
            }
        }

        let get_next_state = |state: BoardState| {
            let current_index_opt = next_states
                .iter()
                .position(|s| s.get_id() == state.get_id());
            assert!(current_index_opt.is_some());

            let next_index = current_index_opt.unwrap() + 1;
            if next_index == next_states.len() {
                (None, None)
            } else {
                (Some(next_states[next_index].clone()), None)
            }
        };

        let (all_states, winner) = print_all_states(next_states[0].clone(), &get_next_state, false);

        assert_eq!(winner, 0);
        assert_eq!(all_states.len(), next_states.len());
        for (index, state) in all_states.iter().enumerate() {
            assert_eq!(state.get_id(), next_states[index].get_id());
        }
    }

    #[test]
    fn human_input() {
        let check_result = |id, input, expected_id_opt: Option<u64>| {
            let (state_opt, eval_opt) = get_next_state_from_user_input(BoardState::from(id), input);
            assert_eq!(state_opt.is_none(), expected_id_opt.is_none());
            assert_eq!(eval_opt, None);
            if let Some(expected_id) = expected_id_opt {
                assert_eq!(state_opt.unwrap().get_id(), expected_id);
            }
        };

        check_result(100382226046, &b"2\n0\n"[..], None);
        check_result(100382226046, &b"\xDF\n \n"[..], None);
        check_result(100382226046, &b"\x82\xe6\n\xDF\n1"[..], Some(100442443391));
        check_result(100382226046, &b"\n\n\n0\n1\n"[..], Some(100442443391));
        check_result(100382226046, &b"0\r\n1\r\n"[..], Some(100442443391));
        check_result(100382226046, &b"2\n0\n3\n1\n"[..], Some(100382229503));
        check_result(100382226046, &b"1 3\n2\n3\n"[..], Some(100382229503));
    }

    #[test]
    fn best_outcome() {
        let init_states = [5057791486, 85065666045].map(BoardState::from);

        let check_result = |id, expected_ids: &[u64], expected_eval| {
            let (state_opt, eval_opt) = get_best_next_state(BoardState::from(id));
            assert!(expected_ids.contains(&state_opt.unwrap().get_id()));
            assert_eq!(eval_opt, Some(expected_eval));
        };

        file_operations::tests::run_in_tempdir(|| {
            generate(&init_states);

            check_result(85065666045, &[85065666046], BoardStateEval::Win);

            for _i in 0..25 {
                check_result(
                    85065666046,
                    &[85066578431, 85125883391, 102408261119],
                    BoardStateEval::Loss,
                );

                let mut state = BoardState::from(85065666045);
                while !state.is_ended() {
                    let (state_opt, eval_opt) = get_best_next_state(state);
                    state = state_opt.unwrap();

                    if state.get_next_player() == 0 {
                        assert_eq!(eval_opt, Some(BoardStateEval::Win));
                    } else {
                        assert_eq!(eval_opt, Some(BoardStateEval::Loss));
                    }
                }
            }

            check_result(5057791486, &[5057794943], BoardStateEval::Draw);
            check_result(5057794943, &[7223777278], BoardStateEval::Draw);

            let mut state = BoardState::from(5057791486);
            for _i in 0..25 {
                let (state_opt, eval_opt) = get_best_next_state(state);
                state = state_opt.unwrap();

                assert!(!state.is_ended());
                assert_eq!(eval_opt, Some(BoardStateEval::Draw));
            }
        });
    }

    #[test]
    fn validate_id() {
        let get_abort_result = |id| {
            std::panic::catch_unwind(|| {
                abort_if_id_is_invalid(id);
            })
        };

        let error_contains_id = |id| {
            let result = get_abort_result(id);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .downcast::<String>()
                .unwrap()
                .contains(&id.to_string()));
        };

        let init_state = BoardState::from(85065666045);

        let err_id = [0, 1, 85065666044, u64::MAX];
        let ok_id = [init_state.get_id(), 85789186557, 59071845884, 67743143411];

        file_operations::tests::run_in_tempdir(|| {
            for &id in err_id.iter().chain(ok_id.iter()) {
                assert!(get_abort_result(id).is_err());
            }

            generate(slice::from_ref(&init_state));

            for id in err_id {
                error_contains_id(id);
            }

            for id in ok_id {
                assert!(get_abort_result(id).is_ok());
            }

            for _i in 0..25 {
                let mut state = init_state.clone();

                while !state.is_ended() {
                    let next_states: Vec<BoardState> = state.get_next_states().collect();
                    state = next_states[fastrand::usize(0..next_states.len())].clone();

                    assert!(get_abort_result(state.get_id()).is_ok());
                }
            }
        });
    }
}
