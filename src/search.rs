use crate::board::{Board, Color};
use crate::r#move::Move;

use crate::eval::Eval;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use std::thread;
use std::time::Duration;

pub type SearchMoves = Vec<(Move, Eval)>;

pub type Time = Option<u64>;
pub type Depth = Option<u16>;
pub type MoveCount = Option<u8>;
pub type Nodes = Option<usize>;
pub type Stopper = Arc<AtomicBool>;

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
        let depth = match depth {
            Some(depth) => depth,
            None => u16::MAX,
        };

        self.minimax(depth, stopper)
    }

    fn minimax(&mut self, depth: u16, stopper: &Stopper) -> SearchMoves {
        let score = match self.turn {
            Color::White => Eval::MIN,
            Color::Black => Eval::MAX,
            Color::Empty => unreachable!(),
        };

        if stopper.load(Ordering::SeqCst) {
            return vec![(Move::null(), score)];
        }

        if depth == 0 {
            return vec![(Move::null(), self.eval())];
        }

        let mut result = Vec::new();

        for mov in self.generate_moves() {
            self.make_move(&mov);
            result.push((mov, self.minimax(depth - 1, stopper)[0].1));
            self.unmake_move(&mov);
        }

        match self.turn {
            Color::White => result.sort_by(|(_, eval1), (_, eval2)| eval2.cmp(eval1)),
            Color::Black => result.sort_by(|(_, eval1), (_, eval2)| eval1.cmp(eval2)),
            Color::Empty => unreachable!(),
        }
        result
    }
}
