use crate::board::Board;
use crate::perft;
use crate::r#move::Move;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::iter::from_fn;

use std::process::exit;

pub fn handle_input(input: String, mut board: Board, stopper: &Arc<AtomicBool>) -> Board {
    let mut input = input[..input.len() - 1].split(" ").peekable();
    match input.next().unwrap() {
        "uci" => {
            println!("id name chess_v4");
            println!("id authon Hugo Lindstom");
            println!("uciok");
        }
        "isready" => println!("readyok"),
        "position" => {
            match input.next().unwrap() {
                "startpos" => board = Board::new(),
                "fen" => {
                    let mut count = 0;
                    let fen = from_fn(|| {
                        if count < 6 {
                            count += 1;
                            input.next()
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                    board = Board::from_fen(fen.clone());
                }
                argument => {
                    println!("\"{argument}\" is not a valid argument to \"position\"!");
                    return board;
                }
            }

            if input.peek().is_some() {
                if input.next().unwrap() == "moves" {
                    for mov in input {
                        board.make_move(&Move::from_string(mov.into()));
                    }
                }
            }
        }
        "go" => match input.next() {
            Some(string) => match string {
                "perft" => match input.next() {
                    Some(string) => match string.parse() {
                        Ok(depth) => {
                            let result = board.perft(depth);
                            println!(
                                "{}\n",
                                result
                                    .iter()
                                    .map(|(mov, count)| format!("{mov}: {count}"))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            );
                            println!("Nodes searched: {}\n", result.values().sum::<i32>());
                        }
                        Err(_) => println!("\n{string}\n is not a valid number!"),
                    },
                    None => println!("perft needs a depth!"),
                },
                argument => {
                    println!("\"{argument}\" is not implemented!");
                }
            },
            None => println!("Not yet implemented!"),
        },
        "print_board" => board.print_board(),
        "stop" => stopper.store(true, Ordering::SeqCst),
        "quit" => exit(0),
        command => println!("\"{command}\" is not implemented!"),
    }

    board
}