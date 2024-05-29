use crate::board::{Board, Color};
use crate::r#move::Move;

use crate::eval::Eval;

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

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
        depth: Depth,
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
        let mut depth = match depth {
            Some(depth) => depth,
            None => u16::MAX,
        };

        match wtime {
            Some(_) => {
                depth = 4;
            }
            None => (),
        }

        let alpha = Eval::MIN + 1;
        let beta = Eval::MAX;

        self.negamax(depth, alpha, beta, stopper)
    }

    fn negamax(&mut self, depth: u16, alpha: Eval, beta: Eval, stopper: &Stopper) -> SearchMoves {
        if stopper.load(Ordering::SeqCst) {
            return vec![(Move::null(), alpha)];
        }

        if depth == 0 {
            return vec![(Move::null(), self.eval())];
        }

        let result = Arc::new(SegQueue::new());
        let break_out = Arc::new(AtomicBool::new(false));
        let alpha = Arc::new(AtomicEval::new(alpha));

        self.generate_moves().par_iter().for_each(|mov| {
            if break_out.load(Ordering::SeqCst) {
                return;
            }
            let mut board = self.clone();
            board.make_move(&mov);
            let score =
                -board.negamax(depth - 1, -beta, -alpha.load(Ordering::SeqCst), stopper)[0].1;

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
