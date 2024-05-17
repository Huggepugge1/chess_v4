use crate::r#move::Move;
use crate::Board;

use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

impl Board {
    pub fn perft(&mut self, depth: i32) -> HashMap<String, i32> {
        if depth == 0 {
            return HashMap::from([(String::new(), 1)]);
        }

        let mut result: HashMap<String, i32> = HashMap::new();
        let moves = self.generate_moves();
        for mut mov in moves {
            self.make_move(&mut mov);
            let count = self.perft(depth - 1).values().sum();
            result.insert(mov.as_string(), count);
            self.unmake_move(&mut mov);
        }

        result
    }

    pub fn perft_test(&mut self, min_depth: i32, max_depth: i32, moves: &mut Vec<String>) {
        if max_depth == 0 {
            self.print_board();
            return;
        }

        if min_depth != max_depth {
            println!("Running perft_test on");
            self.print_board();
            println!("Depth: {max_depth}");
        }

        for depth in min_depth..=max_depth {
            let mut stockfish_result: HashMap<String, i32> = HashMap::new();
            let mut fails: Vec<(String, i32, i32)> = Vec::new();
            let output = match Command::new("stockfish")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(child) => {
                    child
                        .stdin
                        .as_ref()
                        .unwrap()
                        .write(
                            format!("position fen {} moves {}\n", self.fen, moves.join(" "))
                                .as_bytes(),
                        )
                        .unwrap();
                    child
                        .stdin
                        .as_ref()
                        .unwrap()
                        .write(format!("go perft {depth}\n").as_bytes())
                        .unwrap();

                    String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap()
                }
                Err(_e) => String::new(),
            };

            let mut output = output.split("\n");
            output.next().unwrap();

            let mut next = output.next().unwrap();
            while next != "" {
                if next.starts_with("info string") {
                    next = output.next().unwrap();
                    continue;
                }

                let mut line = next.split(": ");
                let mov = line.next().unwrap().to_string();
                let count: i32 = line.next().unwrap().parse().unwrap();
                stockfish_result.insert(mov, count);
                next = output.next().unwrap();
            }

            let result = self.perft(depth);

            for key in stockfish_result.keys() {
                let stockfish_count = stockfish_result.get(key).unwrap();
                let count = match result.get(key) {
                    Some(count) => count.to_owned(),
                    None => 0,
                };

                if *stockfish_count != count {
                    fails.push((key.clone(), count, *stockfish_count));
                }
            }

            for key in result.keys() {
                let count = result.get(key).unwrap();
                match stockfish_result.get(key) {
                    Some(_count) => (),
                    None => fails.push((key.clone(), *count, 0)),
                };
            }

            if fails.len() == 0 && moves.len() == 0 && min_depth != max_depth {
                println!("Performance test OK at depth {depth}");
            } else {
                if min_depth != max_depth {
                    println!("Performance test FAILED at depth {depth}");
                    println!("Fen: {}", self.fen);
                }
                fails.sort();
                moves.push(fails[0].0.clone());
                self.make_move(&Move::from_string(moves[moves.len() - 1].clone()));
                self.perft_test(depth - 1, depth - 1, moves);
                if min_depth != max_depth {
                    println!("moves: {}", moves.join(" "));
                }
                println!("{:?}", fails[0]);
                break;
            }
        }
    }
}
