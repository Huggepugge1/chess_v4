use crate::board::{Board, Color};
use crate::r#move::Move;

use crate::eval::Eval;

use std::collections::HashSet;

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;

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
        let mut max_depth = match max_depth {
            Some(depth) => depth,
            None => u16::MAX,
        };

        match wtime {
            Some(_) => {
                max_depth = 5;
            }
            None => (),
        }

        let mut alpha = match self.turn {
            Color::White => -1000000,
            Color::Black => 1000000,
            Color::Empty => unreachable!(),
        };
        let mut beta = -alpha;

        let mut depth = 1;
        let mut moves = self
            .generate_moves()
            .iter()
            .map(|mov| (mov.clone(), alpha))
            .collect::<Vec<_>>();

        let mut lower_window = self.turn as Eval * 3;
        let mut upper_window = self.turn as Eval * 3;

        while depth <= max_depth {
            if stopper.load(Ordering::SeqCst) {
                return moves;
            }
            let result = self.negamax(depth, alpha, beta, moves.clone(), stopper);
            let best_move = result[0].0;
            let score = result[0].1;

            if best_move == Move::null() || score == alpha {
                if score == beta {
                    lower_window *= 4;
                } else if score == alpha {
                    upper_window *= 4;
                }
                alpha = score - lower_window;
                beta = score + upper_window;
                continue;
            }

            lower_window = self.turn as Eval * 3;
            upper_window = self.turn as Eval * 3;
            alpha = score - lower_window;
            beta = score + upper_window;

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
        stopper: &Stopper,
    ) -> SearchMoves {
        if stopper.load(Ordering::SeqCst) {
            return vec![(Move::null(), alpha)];
        }

        if depth == 0 {
            return vec![(Move::null(), self.eval())];
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

            let score = -board.negamax(depth - 1, -beta, -alpha_clone, moves, stopper)[0].1;

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
            return vec![(Move::null(), beta)];
        }

        let mut final_result = Vec::new();
        while let Some(res) = result.pop() {
            final_result.push(res);
        }

        if final_result.len() == 0 {
            return vec![(Move::null(), alpha.load(Ordering::SeqCst))];
        }

        final_result.sort_by(|(_, eval1), (_, eval2)| eval2.cmp(eval1));
        final_result
    }
}
