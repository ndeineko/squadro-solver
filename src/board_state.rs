use std::{fmt, iter};

// Regular piece progression given [player][piece][piece's position].
const REGULAR_MOVES: [[[usize; 13]; 5]; 2] = [
    [
        [1, 1, 1, 1, 1, 1, 3, 0, 3, 3, 2, 1, 0],
        [3, 0, 3, 3, 2, 1, 1, 1, 1, 1, 1, 1, 0],
        [2, 0, 2, 2, 2, 1, 2, 0, 2, 2, 2, 1, 0],
        [3, 0, 3, 3, 2, 1, 1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 3, 0, 3, 3, 2, 1, 0],
    ],
    [
        [3, 0, 3, 3, 2, 1, 1, 1, 1, 1, 1, 1, 0],
        [1, 1, 1, 1, 1, 1, 3, 0, 3, 3, 2, 1, 0],
        [2, 0, 2, 2, 2, 1, 2, 0, 2, 2, 2, 1, 0],
        [1, 1, 1, 1, 1, 1, 3, 0, 3, 3, 2, 1, 0],
        [3, 0, 3, 3, 2, 1, 1, 1, 1, 1, 1, 1, 0],
    ],
];

// Initial regular piece progression given [player][piece].
const FIRST_MOVES: [[usize; 5]; 2] = [[1, 3, 2, 3, 1], [3, 1, 2, 1, 3]];

// ID is built from the positions of pieces, alternating between
// the two players, and ends with the number of the next player.
const ID_PART_SIZE: [u64; 11] = [12, 12, 12, 12, 11, 11, 12, 12, 12, 12, 2];

// Factor by which each ID part is multiplied.
const ID_PART_FACTOR: [u64; 11] = [
    8671297536, 722608128, 60217344, 5018112, 456192, 41472, 3456, 288, 24, 2, 1,
];

/// State of the game board, including next player and position of pieces
#[derive(Clone)]
pub struct BoardState {
    id: u64,
}

impl BoardState {
    /// Create a new game starting with `first_player`
    pub fn new_game(first_player: usize) -> Self {
        let mut state = Self { id: 0 };
        state.set_next_player(first_player);
        state
    }

    /// Return the ID representing this board state
    pub fn get_id(&self) -> u64 {
        self.id
    }

    /// Return the ID part at the given `index`
    fn get_id_part(&self, index: usize) -> u64 {
        (self.id / ID_PART_FACTOR[index]) % ID_PART_SIZE[index]
    }

    /// Update the ID part at the given `index`
    fn set_id_part(&mut self, index: usize, value: u64) {
        let id_part_factor = ID_PART_FACTOR[index];
        self.id = self.id - (id_part_factor * self.get_id_part(index)) + (id_part_factor * value);
    }

    /// Return the number of the next player
    pub fn get_next_player(&self) -> usize {
        // Shortcut for `self.get_id_part(10) as usize`.
        (self.id as usize) & 1
    }

    /// Set the number of the next player
    fn set_next_player(&mut self, player: usize) {
        self.set_id_part(10, player as u64);
    }

    /// Change the number of the next player
    fn switch_next_player(&mut self) {
        // Shortcut for `self.set_next_player(1 - self.get_next_player())`.
        self.id ^= 1;
    }

    /// Return position of `piece` belonging to `player`
    fn get_piece_position(&self, player: usize, piece: usize) -> usize {
        let mut position = self.get_id_part(piece * 2 + player) as usize;

        // Position in the ID is compressed to store only reachable positions.
        // The actual position must therefore be calculated by adding 1 for each
        // unreachable position.
        if position > 0 {
            let first_move = FIRST_MOVES[player][piece];

            if first_move != 1 {
                position += 1;
            }

            if position > 6 && first_move != 3 {
                position += 1;
            }
        }

        position
    }

    /// Place `piece` belonging to `player` to the given `position`
    fn set_piece_position(&mut self, player: usize, piece: usize, position: usize) {
        let mut position = position;

        // Position in the ID is compressed to store only accessible positions.
        // This is done by taking the actual position and subtracting 1 for each
        // unreachable position.
        if position > 1 {
            let first_move = FIRST_MOVES[player][piece];

            if position > 7 && first_move != 3 {
                position -= 1;
            }

            if first_move != 1 {
                position -= 1;
            }
        }

        self.set_id_part(piece * 2 + player, position as u64);
    }

    /// Is the game over?
    pub fn is_ended(&self) -> bool {
        let last_player = 1 - self.get_next_player();
        let mut movable_pieces: u8 = 0;

        for piece in 0..5 {
            if self.get_piece_position(last_player, piece) < 12 {
                if movable_pieces == 0 {
                    movable_pieces = 1;
                } else {
                    // The game continues as long as more than 1 movable
                    // piece remains.
                    return false;
                }
            }
        }

        true
    }

    /// If two pieces are about to be on the same square, move the first one back
    ///
    /// The piece currently present on the square is moved back to its initial
    /// position or the opposite side.
    /// Return `true` if such a collision occurred.
    fn fix_possible_collision(&mut self, player: usize, piece: usize, position: usize) -> bool {
        if position.is_multiple_of(6) {
            // A collision is impossible when a piece reaches the opposite side
            // or its final position.
            return false;
        }

        let other_player = 1 - player;

        // Get the number of the other player's piece in the perpendicular row.
        let other_piece = if position < 6 {
            position - 1
        } else {
            11 - position
        };

        // Get position of the other player's piece.
        let other_position = self.get_piece_position(other_player, other_piece);

        if other_position.is_multiple_of(6) {
            // A collision is impossible when the other piece is in its initial
            // or final position or on the opposite side.
            return false;
        }
        // The other player's piece hasn't reached the opposite side yet.
        else if other_position < 6 {
            // Are the two pieces colliding ?
            if piece == other_position - 1 {
                // Move the other player's piece back to its initial position.
                self.set_piece_position(other_player, other_piece, 0);
                return true;
            }
        }
        // The other player's piece has already reached the opposite side.
        else {
            // Are the two pieces colliding ?
            if piece == 11 - other_position {
                // Move the other player's piece back to its opposite side.
                self.set_piece_position(other_player, other_piece, 6);
                return true;
            }
        }

        false
    }

    /// Create a new board state in which the next player's `moved_piece` is moved according to the game rules
    ///
    /// Return `None` when `moved_piece` has already reached its final position or is not a valid piece.
    pub fn get_next_state(&self, moved_piece: usize) -> Option<Self> {
        if moved_piece > 4 {
            return None;
        }

        let player = self.get_next_player();
        let mut position = self.get_piece_position(player, moved_piece);
        if position > 11 {
            // The piece is in its final position and can't be moved.
            return None;
        }

        let mut new_state = self.clone();
        new_state.switch_next_player();

        let mut target_position = position + REGULAR_MOVES[player][moved_piece][position];

        // Move the piece, step by step.
        while position != target_position {
            position += 1;

            if new_state.fix_possible_collision(player, moved_piece, position) {
                // When there is a collision, set the target position to the
                // current piece position plus 1.
                target_position = position + 1;
            }
        }

        // Save new position of the piece in `new_state`.
        new_state.set_piece_position(player, moved_piece, position);

        Some(new_state)
    }

    /// Return an iterator over the next board states, assuming the game is not over
    pub fn get_next_states(&self) -> impl Iterator<Item = Self> {
        let current_state = self.clone();
        let mut piece: usize = 0;

        iter::from_fn(move || loop {
            if piece > 4 {
                return None;
            }

            let state_opt = current_state.get_next_state(piece);

            piece += 1;

            // Returning `None` would terminate the iterator prematurely.
            if state_opt.is_some() {
                return state_opt;
            }
        })
    }
}

impl From<u64> for BoardState {
    /// Create a board state from its ID
    fn from(id: u64) -> Self {
        Self { id }
    }
}

impl fmt::Display for BoardState {
    /// Format the board state to display it on a terminal
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let next_player = self.get_next_player();
        let ended = self.is_ended();

        let mut board_arr = [
            [
                ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
                ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ',
            ],
            [
                ' ', ' ', ' ', ' ', ' ', ' ', ' ', '┏', '━', '━', '━', '┳', '━', '━', '━', '┳',
                '━', '━', '━', '┳', '━', '━', '━', '┳', '━', '━', '━', '┓', ' ', ' ', ' ', ' ',
            ],
            [
                ' ', ' ', ' ', ' ', ' ', '■', ' ', '┃', '·', ' ', ' ', '┃', '∵', ' ', ' ', '┃',
                ':', ' ', ' ', '┃', '∵', ' ', ' ', '┃', '·', ' ', ' ', '┃', ' ', '■', ' ', ' ',
            ],
            [
                ' ', ' ', ' ', '┏', '━', '━', '━', '╃', '─', '─', '─', '╀', '─', '─', '─', '╀',
                '─', '─', '─', '╀', '─', '─', '─', '╀', '─', '─', '─', '╄', '━', '━', '━', '┓',
            ],
            [
                ' ', ' ', ' ', '┃', '∵', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', ' ', '│',
                ' ', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', '·', '┃',
            ],
            [
                ' ', ' ', ' ', '┣', '━', '━', '━', '┽', '─', '─', '─', '┼', '─', '─', '─', '┼',
                '─', '─', '─', '┼', '─', '─', '─', '┼', '─', '─', '─', '┾', '━', '━', '━', '┫',
            ],
            [
                ' ', ' ', ' ', '┃', '·', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', ' ', '│',
                ' ', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', '∵', '┃',
            ],
            [
                ' ', ' ', ' ', '┣', '━', '━', '━', '┽', '─', '─', '─', '┼', '─', '─', '─', '┼',
                '─', '─', '─', '┼', '─', '─', '─', '┼', '─', '─', '─', '┾', '━', '━', '━', '┫',
            ],
            [
                ' ', ' ', ' ', '┃', ':', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', ' ', '│',
                ' ', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', ':', '┃',
            ],
            [
                ' ', ' ', ' ', '┣', '━', '━', '━', '┽', '─', '─', '─', '┼', '─', '─', '─', '┼',
                '─', '─', '─', '┼', '─', '─', '─', '┼', '─', '─', '─', '┾', '━', '━', '━', '┫',
            ],
            [
                ' ', ' ', ' ', '┃', '·', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', ' ', '│',
                ' ', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', '∵', '┃',
            ],
            [
                ' ', ' ', ' ', '┣', '━', '━', '━', '┽', '─', '─', '─', '┼', '─', '─', '─', '┼',
                '─', '─', '─', '┼', '─', '─', '─', '┼', '─', '─', '─', '┾', '━', '━', '━', '┫',
            ],
            [
                ' ', ' ', ' ', '┃', '∵', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', ' ', '│',
                ' ', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', ' ', '│', ' ', ' ', '·', '┃',
            ],
            [
                ' ', ' ', ' ', '┗', '━', '━', '━', '╅', '─', '─', '─', '╁', '─', '─', '─', '╁',
                '─', '─', '─', '╁', '─', '─', '─', '╁', '─', '─', '─', '╆', '━', '━', '━', '┛',
            ],
            [
                ' ', ' ', ' ', ' ', ' ', '■', ' ', '┃', ' ', ' ', '∵', '┃', ' ', ' ', '·', '┃',
                ' ', ' ', ':', '┃', ' ', ' ', '·', '┃', ' ', ' ', '∵', '┃', ' ', '■', ' ', ' ',
            ],
            [
                ' ', ' ', ' ', ' ', ' ', ' ', ' ', '┗', '━', '━', '━', '┻', '━', '━', '━', '┻',
                '━', '━', '━', '┻', '━', '━', '━', '┻', '━', '━', '━', '┛', ' ', ' ', ' ', ' ',
            ],
        ];

        // Add pieces of player 0.
        for piece in 0..5 {
            let position = self.get_piece_position(0, piece);

            if position < 6 {
                board_arr[(position + 1) * 2][(piece + 1) * 4 + 5] = '↓';
            } else {
                board_arr[(13 - position) * 2][(piece + 1) * 4 + 5] = '↑';
            }

            // When a piece can be moved next, display its number at the top.
            if !ended && next_player == 0 && position < 12 {
                board_arr[0][(piece + 1) * 4 + 5] = (piece as u8 + b'0') as char;
            }
        }

        // Add pieces of player 1.
        for piece in 0..5 {
            let position = self.get_piece_position(1, piece);

            if position < 6 {
                board_arr[(piece + 2) * 2][position * 4 + 5] = '→';
            } else {
                board_arr[(piece + 2) * 2][(12 - position) * 4 + 5] = '←';
            }

            // When a piece can be moved next, display its number on the left.
            if !ended && next_player == 1 && position < 12 {
                board_arr[(piece + 2) * 2][1] = (piece as u8 + b'0') as char;
            }
        }

        for mut line in board_arr {
            if ended || next_player == 0 {
                // Replace light vertical lines with thick ones.
                for c in line.iter_mut() {
                    *c = match c {
                        '│' => '┃',
                        '╃' | '┽' | '╅' => '╉',
                        '╀' | '┼' | '╁' => '╂',
                        '╄' | '┾' | '╆' => '╊',
                        _ => *c,
                    }
                }
            }

            if ended || next_player == 1 {
                // Replace light horizontal lines with thick ones.
                for c in line.iter_mut() {
                    *c = match c {
                        '─' => '━',
                        '╃' | '╀' | '╄' => '╇',
                        '┽' | '┼' | '┾' => '┿',
                        '╅' | '╁' | '╆' => '╈',
                        '╉' | '╂' | '╊' => '╋',
                        _ => *c,
                    }
                }
            }

            writeln!(f, "{}", String::from_iter(line))?;
        }

        write!(f, "(ID : {})", self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_board() {
        for player in 0..=1 {
            let b = BoardState::new_game(player);

            for piece in 0..5 {
                assert_eq!(b.get_piece_position(0, piece), 0);
                assert_eq!(b.get_piece_position(1, piece), 0);
            }
        }
    }

    #[test]
    fn first_player() {
        for player in 0..=1 {
            assert_eq!(BoardState::new_game(player).get_next_player(), player);
        }
    }

    #[test]
    fn id() {
        let mut b = BoardState::new_game(1);
        assert_eq!(b.get_id(), 1);

        b.set_piece_position(0, 2, 3);
        assert_eq!(b.get_id(), 1 + 912384);

        b.set_piece_position(0, 2, 0);
        assert_eq!(b.get_id(), 1);

        b.set_piece_position(1, 2, 6);
        assert_eq!(b.get_id(), 1 + 207360);

        b.set_next_player(0);
        assert_eq!(b.get_id(), 207360);

        b.set_piece_position(0, 4, 5);
        assert_eq!(b.get_id(), 207360 + 120);

        b.set_piece_position(1, 4, 8);
        assert_eq!(b.get_id(), 207360 + 120 + 14);

        b.set_piece_position(0, 3, 4);
        assert_eq!(b.get_id(), 207360 + 120 + 14 + 10368);
    }

    #[test]
    fn from() {
        for id in [0, 1, 4995120, 104055570117] {
            assert_eq!(BoardState::from(id).get_id(), id);
        }
    }

    #[test]
    fn id_parts() {
        let parts: [u64; 11] = [11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];

        let mut b = BoardState::from(0);

        for (i, &part) in parts.iter().enumerate() {
            b.set_id_part(i, part);
        }

        for (i, &part) in parts.iter().enumerate() {
            assert_eq!(b.get_id_part(i), part);
            b.set_id_part(i, 0);
        }

        assert_eq!(b.get_id(), 0);
    }

    #[test]
    fn next_player() {
        let mut b = BoardState::new_game(1);

        for player in 0..=1 {
            b.set_next_player(player);
            assert_eq!(b.get_next_player(), player);
        }

        b.switch_next_player();
        assert_eq!(b.get_next_player(), 0);
        b.switch_next_player();
        assert_eq!(b.get_next_player(), 1);
    }

    #[test]
    fn piece_position() {
        let mut b = BoardState::new_game(0);

        let positions: [[usize; 5]; 2] = [[0, 6, 12, 9, 9], [7, 1, 12, 1, 6]];

        for (player, pieces_positions) in positions.iter().enumerate() {
            for (piece, &piece_position) in pieces_positions.iter().enumerate() {
                b.set_piece_position(player, piece, piece_position);
            }
        }

        for (player, pieces_positions) in positions.iter().enumerate() {
            for (piece, &piece_position) in pieces_positions.iter().enumerate() {
                assert_eq!(b.get_piece_position(player, piece), piece_position);
                b.set_piece_position(player, piece, 0);
            }
        }

        assert_eq!(b.get_id(), 0);
    }

    #[test]
    fn game_end() {
        let mut b = BoardState::new_game(0);
        assert!(!b.is_ended());

        for i in 0..=2 {
            b.set_piece_position(0, i, 12);
            assert!(!b.is_ended());
        }

        for i in 1..=3 {
            b.set_piece_position(1, i, 12);
            assert!(!b.is_ended());
        }

        b.set_piece_position(1, 0, 12);
        assert!(b.is_ended());

        b.set_piece_position(1, 2, 11);
        assert!(!b.is_ended());

        b.set_next_player(1);
        assert!(!b.is_ended());

        b.set_piece_position(0, 4, 11);
        assert!(!b.is_ended());

        b.set_piece_position(0, 4, 12);
        assert!(b.is_ended());
    }

    #[test]
    fn collisions() {
        let mut b = BoardState::new_game(0);

        b.set_piece_position(0, 0, 2);
        b.set_piece_position(0, 1, 3);
        b.set_piece_position(0, 2, 4);
        b.set_piece_position(0, 3, 10);
        b.set_piece_position(0, 4, 9);

        b.fix_possible_collision(1, 2, 2);
        assert_eq!(b.get_piece_position(0, 0), 2);
        assert_eq!(b.get_piece_position(0, 1), 0);

        b.fix_possible_collision(1, 2, 3);
        assert_eq!(b.get_piece_position(0, 2), 4);

        b.fix_possible_collision(1, 2, 4);
        assert_eq!(b.get_piece_position(0, 3), 10);

        b.fix_possible_collision(1, 2, 5);
        assert_eq!(b.get_piece_position(0, 4), 6);
    }

    #[test]
    fn next_state() {
        let mut b = BoardState::new_game(1);

        b.set_piece_position(0, 0, 1);
        b.set_piece_position(0, 1, 2);
        b.set_piece_position(0, 2, 2);
        b.set_piece_position(0, 3, 7);
        b.set_piece_position(0, 4, 11);

        b.set_piece_position(1, 0, 2);
        b.set_piece_position(1, 1, 12);
        b.set_piece_position(1, 2, 3);
        b.set_piece_position(1, 3, 3);
        b.set_piece_position(1, 4, 7);

        let all_next_states_id: Vec<u64> = b.get_next_states().map(|b| b.get_id()).collect();
        assert_eq!(all_next_states_id.len(), 4);

        // Player 1, piece 0.
        let mut b2 = b.get_next_state(0).expect("Piece 0 should be movable");
        assert!(all_next_states_id.contains(&b2.get_id()));
        assert_eq!(b2.get_piece_position(1, 0), 6);
        b2.set_piece_position(1, 0, 2);
        assert_eq!(b2.get_piece_position(0, 4), 6);
        b2.set_piece_position(0, 4, 11);
        b2.switch_next_player();
        assert_eq!(b2.get_id(), b.get_id());

        // Player 1, piece 1.
        assert!(b.get_next_state(1).is_none());

        // Player 1, piece 2.
        let mut b2 = b.get_next_state(2).expect("Piece 2 should be movable");
        assert!(all_next_states_id.contains(&b2.get_id()));
        assert_eq!(b2.get_piece_position(1, 2), 5);
        b2.set_piece_position(1, 2, 3);
        b2.switch_next_player();
        assert_eq!(b2.get_id(), b.get_id());

        // Player 1, piece 3.
        let mut b2 = b.get_next_state(3).expect("Piece 3 should be movable");
        assert!(all_next_states_id.contains(&b2.get_id()));
        assert_eq!(b2.get_piece_position(1, 3), 4);
        b2.set_piece_position(1, 3, 3);
        b2.switch_next_player();
        assert_eq!(b2.get_id(), b.get_id());

        // Player 1, piece 4.
        let mut b2 = b.get_next_state(4).expect("Piece 4 should be movable");
        assert!(all_next_states_id.contains(&b2.get_id()));
        assert_eq!(b2.get_piece_position(1, 4), 9);
        b2.set_piece_position(1, 4, 7);
        assert_eq!(b2.get_piece_position(0, 3), 6);
        b2.set_piece_position(0, 3, 7);
        b2.switch_next_player();
        assert_eq!(b2.get_id(), b.get_id());

        // Player 1, pieces 5 to 9999.
        for i in 5..10000 {
            assert!(b.get_next_state(i).is_none());
        }

        b.set_next_player(0);

        let all_next_states_id: Vec<u64> = b.get_next_states().map(|b| b.get_id()).collect();
        assert_eq!(all_next_states_id.len(), 5);

        // Player 0, piece 0.
        let mut b2 = b.get_next_state(0).expect("Piece 0 should be movable");
        assert!(all_next_states_id.contains(&b2.get_id()));
        assert_eq!(b2.get_piece_position(0, 0), 2);
        b2.set_piece_position(0, 0, 1);
        b2.switch_next_player();
        assert_eq!(b2.get_id(), b.get_id());

        // Player 0, piece 1.
        let mut b2 = b.get_next_state(1).expect("Piece 1 should be movable");
        assert!(all_next_states_id.contains(&b2.get_id()));
        assert_eq!(b2.get_piece_position(0, 1), 5);
        b2.set_piece_position(0, 1, 2);
        b2.switch_next_player();
        assert_eq!(b2.get_id(), b.get_id());

        // Player 0, piece 2.
        let mut b2 = b.get_next_state(2).expect("Piece 2 should be movable");
        assert!(all_next_states_id.contains(&b2.get_id()));
        assert_eq!(b2.get_piece_position(0, 2), 5);
        b2.set_piece_position(0, 2, 2);
        for piece in 2..=3 {
            assert_eq!(b2.get_piece_position(1, piece), 0);
            b2.set_piece_position(1, piece, 3);
        }
        b2.switch_next_player();
        assert_eq!(b2.get_id(), b.get_id());

        // Player 0, piece 3.
        let mut b2 = b.get_next_state(3).expect("Piece 3 should be movable");
        assert!(all_next_states_id.contains(&b2.get_id()));
        assert_eq!(b2.get_piece_position(0, 3), 8);
        b2.set_piece_position(0, 3, 7);
        b2.switch_next_player();
        assert_eq!(b2.get_id(), b.get_id());

        // Player 0, piece 4.
        let mut b2 = b.get_next_state(4).expect("Piece 4 should be movable");
        assert!(all_next_states_id.contains(&b2.get_id()));
        assert_eq!(b2.get_piece_position(0, 4), 12);
        b2.set_piece_position(0, 4, 11);
        b2.switch_next_player();
        assert_eq!(b2.get_id(), b.get_id());

        // Player 0, pieces 5 to 9999.
        for i in 5..10000 {
            assert!(b.get_next_state(i).is_none());
        }
    }

    #[test]
    fn display() {
        assert_eq!(
            format!("{}", BoardState::from(0)),
            "         0   1   2   3   4      
       ┏━━━┳━━━┳━━━┳━━━┳━━━┓    
     ■ ┃·↓ ┃∵↓ ┃:↓ ┃∵↓ ┃·↓ ┃ ■  
   ┏━━━╉───╂───╂───╂───╂───╊━━━┓
   ┃∵→ ┃   ┃   ┃   ┃   ┃   ┃  ·┃
   ┣━━━╉───╂───╂───╂───╂───╊━━━┫
   ┃·→ ┃   ┃   ┃   ┃   ┃   ┃  ∵┃
   ┣━━━╉───╂───╂───╂───╂───╊━━━┫
   ┃:→ ┃   ┃   ┃   ┃   ┃   ┃  :┃
   ┣━━━╉───╂───╂───╂───╂───╊━━━┫
   ┃·→ ┃   ┃   ┃   ┃   ┃   ┃  ∵┃
   ┣━━━╉───╂───╂───╂───╂───╊━━━┫
   ┃∵→ ┃   ┃   ┃   ┃   ┃   ┃  ·┃
   ┗━━━╉───╂───╂───╂───╂───╊━━━┛
     ■ ┃  ∵┃  ·┃  :┃  ·┃  ∵┃ ■  
       ┗━━━┻━━━┻━━━┻━━━┻━━━┛    
(ID : 0)"
        );

        assert_eq!(
            format!("{}", BoardState::from(1)),
            "                                
       ┏━━━┳━━━┳━━━┳━━━┳━━━┓    
     ■ ┃·↓ ┃∵↓ ┃:↓ ┃∵↓ ┃·↓ ┃ ■  
   ┏━━━╇━━━╇━━━╇━━━╇━━━╇━━━╇━━━┓
 0 ┃∵→ │   │   │   │   │   │  ·┃
   ┣━━━┿━━━┿━━━┿━━━┿━━━┿━━━┿━━━┫
 1 ┃·→ │   │   │   │   │   │  ∵┃
   ┣━━━┿━━━┿━━━┿━━━┿━━━┿━━━┿━━━┫
 2 ┃:→ │   │   │   │   │   │  :┃
   ┣━━━┿━━━┿━━━┿━━━┿━━━┿━━━┿━━━┫
 3 ┃·→ │   │   │   │   │   │  ∵┃
   ┣━━━┿━━━┿━━━┿━━━┿━━━┿━━━┿━━━┫
 4 ┃∵→ │   │   │   │   │   │  ·┃
   ┗━━━╈━━━╈━━━╈━━━╈━━━╈━━━╈━━━┛
     ■ ┃  ∵┃  ·┃  :┃  ·┃  ∵┃ ■  
       ┗━━━┻━━━┻━━━┻━━━┻━━━┛    
(ID : 1)"
        );

        assert_eq!(
            format!("{}", BoardState::from(104055570117)),
            "                                
       ┏━━━┳━━━┳━━━┳━━━┳━━━┓    
     ■ ┃·↑ ┃∵↑ ┃:↑ ┃∵↑ ┃·  ┃ ■  
   ┏━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━┓
   ┃∵← ┃   ┃   ┃   ┃   ┃ ↑ ┃  ·┃
   ┣━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━┫
   ┃·← ┃   ┃   ┃   ┃   ┃   ┃  ∵┃
   ┣━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━┫
   ┃:← ┃   ┃   ┃   ┃   ┃   ┃  :┃
   ┣━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━┫
   ┃·  ┃ ← ┃   ┃   ┃   ┃   ┃  ∵┃
   ┣━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━┫
   ┃∵  ┃ ← ┃   ┃   ┃   ┃   ┃  ·┃
   ┗━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━┛
     ■ ┃  ∵┃  ·┃  :┃  ·┃  ∵┃ ■  
       ┗━━━┻━━━┻━━━┻━━━┻━━━┛    
(ID : 104055570117)"
        );
    }
}
