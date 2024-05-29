use crate::board::Board;
use crate::r#move::Move;

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, Write};
use std::process::{exit, Command, Stdio};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use rand::prelude::*;
use rand::rngs::StdRng;

use rayon::prelude::*;

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

    #[allow(dead_code)]
    pub fn perft_result(&mut self, depth: i32, moves: &Vec<String>) -> Vec<(String, i32, i32)> {
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
                        format!("position fen {} moves {}\n", self.fen, moves.join(" ")).as_bytes(),
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

        fails
    }

    #[allow(dead_code)]
    pub fn perft_test(
        &mut self,
        min_depth: i32,
        max_depth: i32,
        moves: &mut Vec<String>,
    ) -> Result<(), ()> {
        if max_depth == 0 {
            self.print_board();
            return Ok(());
        }

        for depth in min_depth..=max_depth {
            let mut fails = self.perft_result(depth, moves);
            if fails.len() != 0 {
                if min_depth != max_depth {
                    println!("Performance test FAILED at depth {depth}");
                    println!("Fen: {}", self.fen);
                }
                fails.sort();
                moves.push(fails[0].0.clone());
                let current_move = moves[moves.len() - 1].clone();
                self.make_move(&Move::from_string(current_move.clone()));
                let _ = self.perft_test(depth - 1, depth - 1, moves);
                self.unmake_move(&Move::from_string(current_move));
                if min_depth != max_depth {
                    println!("moves: {}", moves.join(" "));
                    println!("chess_v4:  {}", fails[0].1);
                    println!("stockfish: {}", fails[0].2);
                    return Err(());
                }
            }
        }
        return Ok(());
    }

    #[allow(dead_code)]
    pub fn perft_multi_test(file: &str, depth: i32, seed: Option<u64>, index: Option<usize>) {
        let mut file = File::open(file).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let fens = contents.trim();

        let seed = match seed {
            Some(seed) => seed,
            None => random(),
        };

        let mut rng = StdRng::seed_from_u64(seed);
        let mut fens = fens.split("\n").collect::<Vec<_>>();

        fens.shuffle(&mut rng);

        match index {
            Some(index) => {
                println!(
                    "Running perft test at depth {depth} on fen: {}",
                    fens[index]
                );
                let result =
                    Board::from_fen(fens[index].to_string()).perft_test(1, depth, &mut Vec::new());
                match result {
                    Ok(()) => {
                        let _ = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .truncate(true)
                            .open("test_result/perft")
                            .unwrap();
                    }
                    Err(()) => {
                        let mut file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open("test_result/perft")
                            .unwrap();

                        let _ = file.write_fmt(format_args!("{}\n", depth));
                        let _ = file.write_fmt(format_args!("{}\n", seed));
                        let _ = file.write_fmt(format_args!("{}\n", index));
                        exit(1);
                    }
                };
            }
            None => {
                let counter = Arc::new(AtomicUsize::new(0));
                let test_positions = fens.len();
                fens.into_iter()
                    .enumerate()
                    .collect::<Vec<(usize, &str)>>()
                    .par_iter()
                    .for_each(|(i, fen)| {
                        let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
                        println!("{}%", count as f32 * 100f32 / test_positions as f32);
                        println!("Running perft test at depth {depth} on fen: {fen}");
                        let result =
                            Board::from_fen(fen.to_string()).perft_test(1, depth, &mut Vec::new());
                        match result {
                            Ok(()) => {
                                let _ = OpenOptions::new()
                                    .write(true)
                                    .create(true)
                                    .truncate(true)
                                    .open("test_result/perft")
                                    .unwrap();
                            }
                            Err(()) => {
                                let mut file = OpenOptions::new()
                                    .write(true)
                                    .create(true)
                                    .open("test_result/perft")
                                    .unwrap();

                                let _ = file.write_fmt(format_args!("{}\n", depth));
                                let _ = file.write_fmt(format_args!("{}\n", seed));
                                let _ = file.write_fmt(format_args!("{}\n", i));
                                exit(1);
                            }
                        }
                    });
            }
        }
    }

    #[allow(dead_code)]
    pub fn run_perft_multi_test(fen_file: &str, mut depth: i32) {
        let seed;
        let index;

        match File::open("test_result/perft") {
            Ok(result_file) => {
                let reader = BufReader::new(&result_file);

                let metadata = result_file.metadata().unwrap();
                if metadata.len() == 0 {
                    seed = None;
                    index = None;
                } else {
                    let mut lines = reader.lines();

                    depth = lines.next().unwrap().unwrap().trim().parse().unwrap();
                    seed = Some(lines.next().unwrap().unwrap().trim().parse().unwrap());
                    index = Some(lines.next().unwrap().unwrap().trim().parse().unwrap());
                }
            }
            Err(_) => {
                seed = None;
                index = None;
            }
        }

        Self::perft_multi_test(fen_file, depth, seed, index);
    }
}
