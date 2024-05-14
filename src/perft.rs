use crate::Board;

impl Board {
    pub fn perft(&mut self, start_depth: i32, depth: i32) -> i32 {
        if depth == 0 {
            return 1;
        }

        let mut result = 0;
        let moves = self.generate_moves();
        for mut mov in moves {
            self.make_move(&mut mov);
            let move_count = self.perft(start_depth, depth - 1);
            if start_depth == depth {
                println!("{}: {}", mov.as_string(), move_count);
            }
            result += move_count;
            self.unmake_move(&mut mov);
        }

        result
    }
}
