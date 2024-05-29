use crate::board::Board;
use crate::r#move::Move;
use crate::search::Stopper;

use std::sync::atomic::Ordering;
use std::sync::Arc;

use std::thread;

use std::iter::from_fn;

use std::process::exit;

pub fn handle_input(input: String, mut board: Board, stopper: &Stopper) -> Board {
    let mut input = input[..input.len() - 1].split(" ").peekable();
    match input.next().unwrap() {
        "uci" => {
            println!("id name chess_v4");
            println!("id author Hugo Lindström");
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
        "go" => {
            let mut wtime = None;
            let mut btime = None;
            let mut winc = None;
            let mut binc = None;
            let mut moves_to_go = None;
            let mut depth = None;
            let mut nodes = None;
            let mut mate = None;
            let mut movetime = None;

            while input.peek().is_some() {
                match input.next().unwrap() {
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
                            Err(_) => {
                                println!("\"{string}\" is not a valid number!");
                                return board;
                            }
                        },
                        None => {
                            println!("perft needs a depth!");
                            return board;
                        }
                    },
                    "depth" => match input.next() {
                        Some(string) => match string.parse() {
                            Ok(value) => {
                                depth = Some(value);
                            }
                            Err(_) => {
                                println!("\"{string}\" is not a valid number!");
                                return board;
                            }
                        },
                        None => {
                            println!("\"depth\" param needs a depth!");
                            return board;
                        }
                    },
                    "wtime" => match input.next() {
                        Some(string) => match string.parse() {
                            Ok(value) => {
                                wtime = Some(value);
                            }
                            Err(_) => {
                                println!("\"{string}\" is not a valid number!");
                                return board;
                            }
                        },
                        None => {
                            println!("\"wtime\" param needs a wtime!");
                            return board;
                        }
                    },
                    "btime" => match input.next() {
                        Some(string) => match string.parse() {
                            Ok(value) => {
                                btime = Some(value);
                            }
                            Err(_) => {
                                println!("\"{string}\" is not a valid number!");
                                return board;
                            }
                        },
                        None => {
                            println!("\"btime\" param needs a btime!");
                            return board;
                        }
                    },
                    "winc" => match input.next() {
                        Some(string) => match string.parse() {
                            Ok(value) => {
                                winc = Some(value);
                            }
                            Err(_) => {
                                println!("\"{string}\" is not a valid number!");
                                return board;
                            }
                        },
                        None => {
                            println!("\"winc\" param needs a btime!");
                            return board;
                        }
                    },
                    "binc" => match input.next() {
                        Some(string) => match string.parse() {
                            Ok(value) => {
                                binc = Some(value);
                            }
                            Err(_) => {
                                println!("\"{string}\" is not a valid number!");
                                return board;
                            }
                        },
                        None => {
                            println!("\"binc\" param needs a btime!");
                            return board;
                        }
                    },
                    argument => {
                        println!("\"{argument}\" is not implemented!");
                        return board;
                    }
                }
            }
            let mut board = board.clone();
            let stopper = Arc::clone(stopper);
            thread::spawn(move || {
                let result = board.search(
                    wtime,
                    btime,
                    winc,
                    binc,
                    moves_to_go,
                    depth,
                    nodes,
                    mate,
                    movetime,
                    &stopper,
                )[0]
                .0
                .as_string();
                match result.as_str() {
                    "`1`1" => println!("Did not find any legal moves!"),
                    mov => println!("bestmove {mov}"),
                }
            });
        }
        "print_board" => board.print_board(),
        "stop" => stopper.store(true, Ordering::SeqCst),
        "quit" => exit(0),
        command => println!("\"{command}\" is not implemented!"),
    }

    board
}
