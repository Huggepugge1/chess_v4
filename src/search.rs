use crate::board::{Board, Color};
use crate::piece::PieceType;
use crate::r#move::Move;

use crate::eval::Eval;

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use std::collections::HashMap;

use std::thread;
use std::time::Duration;

use rayon::prelude::*;

use crossbeam::queue::SegQueue;

pub type SearchMoves = Vec<(Move, Eval)>;

pub type Time = Option<u64>;
pub type Depth = Option<u16>;
pub type MoveCount = Option<u8>;
pub type Nodes = Option<usize>;
pub type Stopper = Arc<AtomicBool>;
pub type AtomicEval = AtomicI64;

#[derive(Debug, Clone)]
struct TranspositionEntry {
    depth: u16,
    result: SearchMoves,
}

type TranspositionTable = HashMap<Board, TranspositionEntry>;

pub fn timer(time: u64, stopper: &Stopper) {
    thread::sleep(Duration::from_millis(time));
    stopper.store(true, Ordering::SeqCst);
}

impl Board {
    pub fn search(
        &mut self,
        wtime: Time,
        btime: Time,
        winc: Time,
        binc: Time,
        moves_to_go: MoveCount,
        max_depth: Depth,
        nodes: Nodes,
        mate: MoveCount,
        movetime: Time,
        stopper: &Stopper,
    ) -> SearchMoves {
        stopper.store(false, Ordering::SeqCst);
        let time = match movetime {
            Some(time) => Some(time),
            None => None,
        };
        match time {
            Some(time) => {
                let stopper_clone = Arc::clone(stopper);
                thread::spawn(move || timer(time, &stopper_clone));
            }
            None => (),
        }
        let max_depth = match max_depth {
            Some(depth) => depth,
            None => u16::MAX,
        };

        match self.turn {
            Color::White => match wtime {
                Some(white_time) => {
                    let stopper_clone = Arc::clone(stopper);
                    thread::spawn(move || timer(white_time / 40, &stopper_clone));
                }
                None => (),
            },
            Color::Black => match btime {
                Some(black_time) => {
                    let stopper_clone = Arc::clone(stopper);
                    thread::spawn(move || timer(black_time / 40, &stopper_clone));
                }
                None => (),
            },
            Color::Empty => unreachable!(),
        }

        let mut alpha = -Eval::MAX + 100;
        let mut beta = -alpha;

        let mut depth = 1;

        let mut transposition_table = Arc::new(Mutex::new(HashMap::new()));
        let mut moves = self
            .generate_moves()
            .iter()
            .map(|mov| (mov.clone(), beta))
            .collect::<Vec<_>>();

        let mut lower_window = 25;
        let mut upper_window = 25;

        let mut previous_score = alpha;

        while depth <= max_depth {
            if stopper.load(Ordering::SeqCst) {
                return moves;
            }

            let transposition_clone = transposition_table.lock().unwrap().clone();

            let result = self.negamax(
                depth,
                alpha,
                beta,
                moves.clone(),
                Arc::clone(&transposition_table),
                stopper,
            );
            let best_move = result[0].0;
            let score = result[0].1;

            if best_move == Move::null() || score == alpha {
                println!(
                    "{:?} score: {} alpha: {} beta: {} lower: {} upper: {}",
                    best_move.as_string(),
                    score,
                    alpha,
                    beta,
                    lower_window,
                    upper_window
                );
                // fail low
                if score == beta {
                    upper_window *= 4;

                // fail high
                } else if score == alpha {
                    lower_window *= 4;
                }

                alpha = previous_score - lower_window;
                beta = previous_score + upper_window;

                transposition_table = Arc::new(Mutex::new(transposition_clone));
                continue;
            }

            lower_window = 25;
            upper_window = 25;

            alpha = score - lower_window;
            beta = score + upper_window;

            previous_score = score;

            moves = result;
            depth += 1;
        }

        moves
    }

    fn negamax(
        &mut self,
        depth: u16,
        alpha: Eval,
        beta: Eval,
        moves: SearchMoves,
        transposition_table: Arc<Mutex<TranspositionTable>>,
        stopper: &Stopper,
    ) -> SearchMoves {
        if moves.len() == 0 {
            return vec![(Move::null(), beta)];
        }

        if stopper.load(Ordering::SeqCst) {
            return vec![(Move::null(), alpha)];
        }

        if depth == 0 {
            return self.quiescence_search(alpha, beta, moves, stopper);
        }

        match transposition_table.lock().unwrap().get(self) {
            Some(entry) => {
                if entry.depth >= depth + 1 {
                    if entry.result[0].1 <= alpha {
                        return vec![(Move::null(), alpha)];
                    }
                    if entry.result[0].1 >= beta {
                        return vec![(Move::null(), beta)];
                    }
                    return entry.result.clone();
                }
            }
            None => (),
        }

        let result = Arc::new(SegQueue::new());
        let break_out = Arc::new(AtomicBool::new(false));
        let alpha = Arc::new(AtomicEval::new(alpha));

        moves.par_iter().for_each(|(mov, _)| {
            if break_out.load(Ordering::SeqCst) {
                return;
            }
            let mut board = self.clone();
            board.make_move(&mov);
            let alpha_clone = alpha.load(Ordering::SeqCst);
            let moves = board
                .generate_moves()
                .iter()
                .map(|mov| (mov.clone(), alpha_clone))
                .collect::<Vec<_>>();

            let score = -board.negamax(
                depth - 1,
                -beta,
                -alpha_clone,
                moves,
                Arc::clone(&transposition_table),
                stopper,
            )[0]
            .1;

            board.unmake_move(&mov);

            if score >= beta {
                break_out.store(true, Ordering::SeqCst);
                return;
            }

            if score > alpha.load(Ordering::SeqCst) {
                alpha.store(score, Ordering::SeqCst);
            }

            result.push((mov.clone(), score));
        });

        if break_out.load(Ordering::SeqCst) {
            let result = vec![(Move::null(), beta)];
            return result;
        }

        let mut final_result = Vec::new();
        while let Some(res) = result.pop() {
            final_result.push(res);
        }

        if final_result.len() == 0 {
            let result = vec![(Move::null(), alpha.load(Ordering::SeqCst))];
            return result;
        }

        final_result.sort_by(|(_, eval1), (_, eval2)| eval2.cmp(eval1));
        transposition_table.lock().unwrap().insert(
            self.clone(),
            TranspositionEntry {
                depth,
                result: final_result.clone(),
            },
        );
        final_result
    }

    fn quiescence_search(
        &mut self,
        mut alpha: Eval,
        beta: Eval,
        moves: SearchMoves,
        stopper: &Stopper,
    ) -> SearchMoves {
        if stopper.load(Ordering::SeqCst) {
            return vec![(Move::null(), alpha)];
        }

        let stand_pat = self.eval();

        if stand_pat >= beta {
            return vec![(Move::null(), beta)];
        }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let result = Arc::new(SegQueue::new());
        let break_out = Arc::new(AtomicBool::new(false));
        let alpha = Arc::new(AtomicEval::new(alpha));

        moves.par_iter().for_each(|(mov, _)| {
            if break_out.load(Ordering::SeqCst)
                || self.get_piece(mov.end_square).typ == PieceType::Empty
            {
                return;
            }
            let mut board = self.clone();
            board.make_move(&mov);
            let alpha_clone = alpha.load(Ordering::SeqCst);
            let moves = board
                .generate_moves()
                .iter()
                .map(|mov| (mov.clone(), alpha_clone))
                .collect::<Vec<_>>();

            let score = -board.quiescence_search(-beta, -alpha_clone, moves, stopper)[0].1;

            board.unmake_move(&mov);

            if score >= beta {
                break_out.store(true, Ordering::SeqCst);
                return;
            }

            if score > alpha.load(Ordering::SeqCst) {
                alpha.store(score, Ordering::SeqCst);
            }

            result.push((mov.clone(), score));
        });

        if break_out.load(Ordering::SeqCst) {
            let result = vec![(Move::null(), beta)];
            return result;
        }

        let mut final_result = Vec::new();
        while let Some(res) = result.pop() {
            final_result.push(res);
        }

        if final_result.len() == 0 {
            let result = vec![(Move::null(), alpha.load(Ordering::SeqCst))];
            return result;
        }

        final_result.sort_by(|(_, eval1), (_, eval2)| eval2.cmp(eval1));
        final_result
    }
}
