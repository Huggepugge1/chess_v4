extern crate pgnparse;
use pgnparse::parser::*;

use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use itertools::Itertools;

use rayon::prelude::*;

pub fn get_fens_from_pgn(pgn_string: String) -> Vec<String> {
    let mut fens = Vec::new();

    let result = parse_pgn_to_rust_struct(pgn_string.clone());

    for i in result.moves {
        fens.push(i.fen_after);
    }

    fens
}

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

    games.par_iter().for_each(|game| {
        let result = get_fens_from_pgn(game.to_string());

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("fens.txt")
            .unwrap();

        let _ = file.write_fmt(format_args!("{}\n", result.join("\n")));

        let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
        if count % 1000 == 0 {
            println!("{}%", count as f32 * 100f32 / games.len() as f32);
        }
    });
}
