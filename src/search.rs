use crate::board::{Board, Color};
use crate::r#move::Move;

use crate::eval::Eval;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::collections::HashMap;

use std::thread;
use std::time::Duration;

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

        let mut alpha = Eval::MIN;
        let mut beta = Eval::MAX;

        let mut depth = 1;

        let mut transposition_table = HashMap::new();
        let mut moves = self.generate_search_moves();

        let mut lower_window = Eval::from(25i64);
        let mut upper_window = Eval::from(25i64);

        let mut previous_score = alpha;

        while depth <= max_depth {
            if stopper.load(Ordering::SeqCst) {
                return moves;
            }

            let transposition_clone = transposition_table.clone();

            let result = self.negamax(
                depth,
                alpha,
                beta,
                moves.clone(),
                &mut transposition_table,
                stopper,
            );
            let best_move = result[0].mov;
            let score = result[0].eval;

            if best_move == Move::null() || score == alpha {
                if score == beta {
                    upper_window = upper_window * Eval::from(4i64);
                } else if score == alpha {
                    lower_window = lower_window * Eval::from(4i64);
                }

                alpha = previous_score - lower_window;
                beta = previous_score + upper_window;

                transposition_table = transposition_clone;
                continue;
            }

            lower_window = Eval::from(25i64);
            upper_window = Eval::from(25i64);

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
        mut alpha: Eval,
        beta: Eval,
        moves: Vec<SearchMove>,
        transposition_table: &mut TranspositionTable,
        stopper: &Stopper,
    ) -> Vec<SearchMove> {
        if moves.len() == 0 {
            return vec![SearchMove {
                mov: Move::null(),
                eval: self.eval(moves.len()),
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

        match transposition_table.get(self) {
            Some(entry) => {
                if entry.depth >= depth + 1 {
                    if entry.result[0].eval.mate != None {
                        return entry.result.clone();
                    }
                    if entry.result[0].eval <= alpha {
                        return vec![SearchMove {
                            mov: Move::null(),
                            eval: alpha.into(),
                        }];
                    }
                    if entry.result[0].eval >= beta {
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

        let mut result = Vec::new();

        for SearchMove { mov, eval: _ } in moves {
            let mut board = self.clone();

            board.make_move(&mov);
            let moves = board.generate_search_moves();

            let score = -board.negamax(
                depth - 1,
                -beta,
                -alpha,
                moves,
                transposition_table,
                stopper,
            )[0]
            .eval;

            board.unmake_move(&mov);

            if score >= beta {
                let result = vec![SearchMove {
                    mov: Move::null(),
                    eval: beta.into(),
                }];
                return result;
            }

            if score > alpha {
                alpha = score;
            }

            result.push(SearchMove {
                mov: mov.clone(),
                eval: score,
            });
        }

        if result.len() == 0 {
            let result = vec![SearchMove {
                mov: Move::null(),
                eval: alpha,
            }];
            return result;
        }

        result.sort_by(
            |SearchMove {
                 mov: _,
                 eval: eval1,
             },
             SearchMove {
                 mov: _,
                 eval: eval2,
             }| { eval2.cmp(eval1) },
        );

        transposition_table.insert(
            self.clone(),
            TranspositionEntry {
                depth,
                result: result.clone(),
            },
        );

        result
    }

    fn quiescence_search(
        &mut self,
        mut alpha: Eval,
        beta: Eval,
        moves: Vec<SearchMove>,
        stopper: &Stopper,
    ) -> Vec<SearchMove> {
        if stopper.load(Ordering::SeqCst) {
            return vec![SearchMove {
                mov: Move::null(),
                eval: beta.into(),
            }];
        }

        let stand_pat = self.eval(moves.len());

        if stand_pat.mate != None {
            return vec![SearchMove {
                mov: Move::null(),
                eval: Eval {
                    score: stand_pat.score,
                    mate: Some(stand_pat.mate.unwrap() + 1),
                },
            }];
        }

        if stand_pat >= beta {
            return vec![SearchMove {
                mov: Move::null(),
                eval: beta.into(),
            }];
        }

        if alpha < stand_pat {
            alpha = stand_pat;
        }

        let mut result = Vec::new();

        for SearchMove { mov, eval: _ } in moves {
            if self.is_quiet(&mov) {
                continue;
            }

            self.make_move(&mov);
            let moves = self.generate_search_moves();

            let score = -self.quiescence_search(-beta, -alpha, moves, stopper)[0].eval;

            self.unmake_move(&mov);

            if score >= beta {
                let result = vec![SearchMove {
                    mov: Move::null(),
                    eval: beta,
                }];
                return result;
            }

            if score > alpha {
                alpha = score;
            }

            result.push(SearchMove {
                mov: mov.clone(),
                eval: score,
            });
        }

        if result.len() == 0 {
            let result = vec![SearchMove {
                mov: Move::null(),
                eval: alpha,
            }];
            return result;
        }

        result.sort_by(
            |SearchMove {
                 mov: _,
                 eval: eval1,
             },
             SearchMove {
                 mov: _,
                 eval: eval2,
             }| { eval2.cmp(eval1) },
        );

        result
    }
}
