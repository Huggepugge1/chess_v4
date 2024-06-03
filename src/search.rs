use crate::board::{Board, Color};
use crate::r#move::Move;

use crate::eval::Eval;

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use std::collections::HashMap;

use std::thread;
use std::time::Duration;

use rayon::prelude::*;

use crossbeam::queue::SegQueue;

#[derive(Debug, Clone)]
pub struct SearchMove {
    pub mov: Move,
    pub eval: Eval,
}

pub type Time = Option<u64>;
pub type Depth = Option<u16>;
pub type MoveCount = Option<u8>;
pub type Nodes = Option<usize>;
pub type Stopper = Arc<AtomicBool>;
pub type AtomicEval = AtomicI64;

#[derive(Debug, Clone)]
struct TranspositionEntry {
    depth: u16,
    result: Vec<SearchMove>,
}

type TranspositionTable = HashMap<Board, TranspositionEntry>;

pub fn timer(time: u64, stopper: &Stopper) {
    thread::sleep(Duration::from_millis(time));
    stopper.store(true, Ordering::SeqCst);
}

impl Board {
    fn generate_search_moves(&mut self) -> Vec<SearchMove> {
        self.generate_moves()
            .iter()
            .map(|mov| SearchMove {
                mov: mov.clone(),
                eval: Eval::from(0i64),
            })
            .collect()
    }

    pub fn search(
        &mut self,
        wtime: Time,
        btime: Time,
        _winc: Time,
        _binc: Time,
        _moves_to_go: MoveCount,
        max_depth: Depth,
        _nodes: Nodes,
        _mate: MoveCount,
        movetime: Time,
        stopper: &Stopper,
    ) -> Vec<SearchMove> {
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

        let mut alpha = Eval::MIN.score;
        let mut beta = Eval::MAX.score;

        let mut depth = 1;

        let mut transposition_table = Arc::new(Mutex::new(HashMap::new()));
        let mut moves = self.generate_search_moves();

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
            let best_move = result[0].mov;
            let score = result[0].eval;

            if best_move == Move::null() || score.score == alpha {
                if score.score == beta {
                    upper_window *= 4;
                } else if score.score == alpha {
                    lower_window *= 4;
                }

                alpha = previous_score - lower_window;
                beta = previous_score + upper_window;

                transposition_table = Arc::new(Mutex::new(transposition_clone));
                continue;
            }

            lower_window = 25;
            upper_window = 25;

            alpha = score.score - lower_window;
            beta = score.score + upper_window;

            previous_score = score.score;

            moves = result;
            depth += 1;
        }

        moves
    }

    fn negamax(
        &mut self,
        depth: u16,
        alpha: i64,
        beta: i64,
        moves: Vec<SearchMove>,
        transposition_table: Arc<Mutex<TranspositionTable>>,
        stopper: &Stopper,
    ) -> Vec<SearchMove> {
        if moves.len() == 0 {
            return vec![SearchMove {
                mov: Move::null(),
                eval: self.eval(),
            }];
        }

        if stopper.load(Ordering::SeqCst) {
            return vec![SearchMove {
                mov: Move::null(),
                eval: alpha.into(),
            }];
        }

        if depth == 0 {
            return self.quiescence_search(alpha, beta, moves, stopper);
        }

        match transposition_table.lock().unwrap().get(self) {
            Some(entry) => {
                if entry.depth >= depth + 1 {
                    if entry.result[0].eval.score <= alpha {
                        return vec![SearchMove {
                            mov: Move::null(),
                            eval: alpha.into(),
                        }];
                    }
                    if entry.result[0].eval.score >= beta {
                        return vec![SearchMove {
                            mov: Move::null(),
                            eval: beta.into(),
                        }];
                    }
                    return entry.result.clone();
                }
            }
            None => (),
        }

        let result = Arc::new(SegQueue::new());
        let break_out = Arc::new(AtomicBool::new(false));
        let alpha = Arc::new(AtomicEval::new(alpha));

        moves.par_iter().for_each(|SearchMove { mov, eval: _ }| {
            if break_out.load(Ordering::SeqCst) {
                return;
            }
            let mut board = self.clone();
            board.make_move(&mov);
            let alpha_clone = alpha.load(Ordering::SeqCst);
            let moves = board.generate_search_moves();

            let mut score = -board.negamax(
                depth - 1,
                -beta,
                -alpha_clone,
                moves,
                Arc::clone(&transposition_table),
                stopper,
            )[0]
            .eval;

            board.unmake_move(&mov);

            let mut mate = false;
            match score.mate {
                Some(mate_ply) => {
                    mate = true;
                    score.mate = Some(mate_ply + 1);
                }
                None => (),
            }

            if score.score >= beta && !mate {
                break_out.store(true, Ordering::SeqCst);
                return;
            }

            if score.score > alpha.load(Ordering::SeqCst) && !mate {
                alpha.store(score.score, Ordering::SeqCst);
            }

            result.push(SearchMove {
                mov: mov.clone(),
                eval: score,
            });
        });

        if break_out.load(Ordering::SeqCst) {
            let result = vec![SearchMove {
                mov: Move::null(),
                eval: beta.into(),
            }];
            return result;
        }

        let mut final_result = Vec::new();
        while let Some(res) = result.pop() {
            final_result.push(res);
        }

        if final_result.len() == 0 {
            let result = vec![SearchMove {
                mov: Move::null(),
                eval: alpha.load(Ordering::SeqCst).into(),
            }];
            return result;
        }

        final_result.sort_by(
            |SearchMove {
                 mov: _,
                 eval: eval1,
             },
             SearchMove {
                 mov: _,
                 eval: eval2,
             }| { eval2.cmp(eval1) },
        );

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
        mut alpha: i64,
        beta: i64,
        moves: Vec<SearchMove>,
        stopper: &Stopper,
    ) -> Vec<SearchMove> {
        if stopper.load(Ordering::SeqCst) {
            return vec![SearchMove {
                mov: Move::null(),
                eval: beta.into(),
            }];
        }

        let stand_pat = self.eval();

        if stand_pat.mate != None {
            return vec![SearchMove {
                mov: Move::null(),
                eval: Eval {
                    score: stand_pat.score,
                    mate: Some(stand_pat.mate.unwrap() + 1),
                },
            }];
        }

        let stand_pat = stand_pat.score;

        if stand_pat >= beta {
            return vec![SearchMove {
                mov: Move::null(),
                eval: beta.into(),
            }];
        }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let result = Arc::new(SegQueue::new());
        let break_out = Arc::new(AtomicBool::new(false));
        let alpha = Arc::new(AtomicEval::new(alpha));

        moves.par_iter().for_each(|SearchMove { mov, eval: _ }| {
            let mut board = self.clone();
            if break_out.load(Ordering::SeqCst) || board.is_quiet(mov) {
                return;
            }

            let zobrist = board.zobrist;
            board.make_move(&mov);
            board.unmake_move(&mov);
            if zobrist != board.zobrist {
                println!("zobrist mismatch");
                for (index, zobrist_hash) in self.zobrist_array.iter().enumerate() {
                    if zobrist ^ zobrist_hash == board.zobrist {
                        println!("index: {}", index);
                    }
                }
            }

            board.make_move(&mov);
            let alpha_clone = alpha.load(Ordering::SeqCst);
            let moves = board.generate_search_moves();

            let mut score = -board.quiescence_search(-beta, -alpha_clone, moves, stopper)[0].eval;

            board.unmake_move(&mov);

            let mut mate = false;
            match score.mate {
                Some(mate_ply) => {
                    mate = true;
                    score.mate = Some(mate_ply + 1);
                }
                None => (),
            }

            if score.score >= beta && !mate {
                break_out.store(true, Ordering::SeqCst);
                return;
            }

            if score.score > alpha.load(Ordering::SeqCst) && !mate {
                alpha.store(score.score, Ordering::SeqCst);
            }

            result.push(SearchMove {
                mov: mov.clone(),
                eval: score,
            });
        });

        if break_out.load(Ordering::SeqCst) {
            let result = vec![SearchMove {
                mov: Move::null(),
                eval: beta.into(),
            }];
            return result;
        }

        let mut final_result = Vec::new();
        while let Some(res) = result.pop() {
            final_result.push(res);
        }

        if final_result.len() == 0 {
            let result = vec![SearchMove {
                mov: Move::null(),
                eval: alpha.load(Ordering::SeqCst).into(),
            }];
            return result;
        }

        final_result.sort_by(
            |SearchMove {
                 mov: _,
                 eval: eval1,
             },
             SearchMove {
                 mov: _,
                 eval: eval2,
             }| { eval2.cmp(eval1) },
        );

        final_result
    }
}
