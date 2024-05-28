extern crate pgnparse;
use pgnparse::parser::*;

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use itertools::Itertools;

use rayon::prelude::*;

#[allow(dead_code)]
pub fn get_fens_from_pgn(pgn_string: String, seen: &Mutex<HashSet<String>>) {
    let mut seen = seen.lock().unwrap();
    let result = parse_pgn_to_rust_struct(pgn_string.clone());

    for i in result.moves {
        if !seen.contains(&i.fen_after[..(i.fen_after.len() - 3)]) {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open("fens.txt")
                .unwrap();

            let _ = file.write_fmt(format_args!("{}\n", i.fen_after));
            seen.insert(i.fen_after[..(i.fen_after.len() - 3)].to_string());
        }
    }
}

#[allow(dead_code)]
pub fn convert_pgn_from_file(pgn_file: &str) {
    let file = File::open(pgn_file).unwrap();

    let mut buf: Vec<u8> = Vec::new();
    let _ = BufReader::new(file).read_to_end(&mut buf).unwrap();

    let games = String::from_utf8(buf)
        .unwrap()
        .split("\n\n")
        .into_iter()
        .chunks(2)
        .into_iter()
        .map(|chunk| chunk.collect::<Vec<_>>().join("\n\n"))
        .into_iter()
        .collect_vec();

    let counter = Arc::new(AtomicUsize::new(0));
    let seen = Arc::new(Mutex::new(HashSet::new()));

    games.par_iter().for_each(|game| {
        get_fens_from_pgn(game.to_string(), &seen);

        let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
        if count % 1000 == 0 {
            println!("{}%", count as f32 * 100f32 / games.len() as f32);
        }
    });
}
