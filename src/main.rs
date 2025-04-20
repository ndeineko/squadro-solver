mod board_state;
mod file_operations;
mod generate;
mod play;

use clap::{Parser, Subcommand, ValueEnum};

use crate::board_state::BoardState;
use crate::generate::generate;
use crate::play::play;

/// Solver for the Squadro board game
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    /// Play a game
    Play {
        /// Player controled by human
        ///
        /// If not specified, the computer will play against itself.
        #[arg(short, long, value_enum, value_name = "PLAYER")]
        player: Option<Player>,

        /// Player who makes the first move
        ///
        /// If not specified, the first player is selected at random.
        #[arg(short, long, value_enum, value_name = "PLAYER")]
        first: Option<Player>,

        /// Initial board state ID
        ///
        /// The first player cannot be specified since it is already included in the ID.
        #[arg(short, long, conflicts_with = "first")]
        id: Option<u64>,

        /// Show evaluation of position when computer plays
        #[arg(short, long)]
        eval: bool,
    },

    /// Generate game data (WARNING : memory-intensive and time-consuming process)
    Generate,
}

#[repr(usize)]
#[derive(Clone, ValueEnum)]
enum Player {
    /// Top player
    Top = 0,

    /// Left player
    Left = 1,
}

fn main() {
    match Cli::parse().command {
        SubCommand::Play {
            player,
            first,
            id,
            eval,
        } => {
            play(
                // If `id` is provided, play from that board state ID.
                // Otherwise, if `first` is provided, play a game from
                // the initial board state, with the given first player.
                // When neither of these arguments is provided, play a game
                // from the initial board state, with a random first player.
                id.unwrap_or_else(|| {
                    BoardState::new_game(first.unwrap_or_else(|| {
                        if fastrand::bool() {
                            Player::Left
                        } else {
                            Player::Top
                        }
                    }) as usize)
                    .get_id()
                }),
                player.map(|p| p as usize),
                eval,
            );
        }
        SubCommand::Generate => {
            generate(&([Player::Top, Player::Left].map(|p| BoardState::new_game(p as usize))));
        }
    }
}
