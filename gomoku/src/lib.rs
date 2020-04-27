#![allow(dead_code)]
//#![deny(missing_docs)]
//! Gomoku

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Chessman {
    Black,
    White,
}

#[derive(Debug)]
pub enum Error {
    OutOfBounds,
    InvalidPos,
    OpNotAllowed,
    NotYourTurn,
}

pub type Result<T> = std::result::Result<T, Error>;

const LINE_COUNT: usize = 15;

pub struct Board {
    pub states: [[Option<Chessman>; LINE_COUNT]; LINE_COUNT],
    last: Chessman,
}

impl Default for Board {
    fn default() -> Board {
        Board {
            last: Chessman::White,
            states: [[None; LINE_COUNT]; LINE_COUNT],
        }
    }
}

impl Board {
    /// Dump states of all chessmen
    /// None mapped to 0, black mapped to 1,
    /// and white mapped to 2
    pub fn dump_states(&self) -> Vec<Vec<u8>> {
        self.states
            .iter()
            .map(|l| {
                l.iter()
                    .map(|s| match s {
                        None => 0,
                        Some(Chessman::Black) => 1,
                        Some(Chessman::White) => 2,
                    })
                    .collect()
            })
            .collect()
    }

    /// Put a chessman on the board
    pub fn put_piece(&mut self, c: Chessman, x: usize, y: usize) -> Result<bool> {
        self.check(c, x, y)?;
        self.states[x][y] = Some(c);
        self.last = c;
        let w = self.wins();
        Ok(w)
    }

    fn check(&self, c: Chessman, x: usize, y: usize) -> Result<()> {
        if x >= LINE_COUNT || y >= LINE_COUNT {
            return Err(Error::OutOfBounds);
        }
        if self.last == c {
            return Err(Error::NotYourTurn);
        }
        if self.states[x][y].is_some() {
            return Err(Error::InvalidPos);
        }
        if Chessman::Black == c {
            // TODO check
        }
        Ok(())
    }

    fn wins(&self) -> bool {
        for x in 0..LINE_COUNT - 4 {
            for y in 0..LINE_COUNT - 4 {
                if self.states[x][y] != Some(self.last) {
                    continue;
                }
                // check right
                let mut win = true;
                for i in x..x + 5 {
                    if self.states[i][y] != Some(self.last) {
                        win = false;
                        break;
                    }
                }
                if win {
                    return true;
                }

                // check down
                let mut win = true;
                for i in y..y + 5 {
                    if self.states[x][i] != Some(self.last) {
                        win = false;
                        break;
                    }
                }
                if win {
                    return true;
                }

                // check right-down
                let mut win = true;
                for i in 0..5 {
                    if self.states[x + i][y + i] != Some(self.last) {
                        win = false;
                        break;
                    }
                }
                if win {
                    return true;
                }
                // check left-down
                let mut win = true;
                for i in 0..x {
                    if self.states[x - i][y + i] != Some(self.last) {
                        win = false;
                        break;
                    }
                }
                if win {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(board: &mut Board, c: Chessman, x: usize, y: usize, win: bool) {
        let w = board.put_piece(c, x, y).unwrap();
        assert_eq!(win, w);
    }

    #[test]
    fn it_works() {
        let mut board = Board::default();
        put(&mut board, Chessman::Black, 7, 7, false);
        put(&mut board, Chessman::White, 6, 6, false);
        put(&mut board, Chessman::Black, 6, 7, false);
        put(&mut board, Chessman::White, 5, 7, false);
        put(&mut board, Chessman::Black, 5, 6, false);
        put(&mut board, Chessman::White, 7, 5, false);
        put(&mut board, Chessman::Black, 8, 4, false);
        put(&mut board, Chessman::White, 4, 8, false);
        put(&mut board, Chessman::Black, 3, 9, false);
        put(&mut board, Chessman::White, 6, 8, false);
        put(&mut board, Chessman::Black, 8, 7, false);
        put(&mut board, Chessman::White, 5, 8, false);
        put(&mut board, Chessman::Black, 3, 8, false);
        put(&mut board, Chessman::White, 4, 6, false);
        put(&mut board, Chessman::Black, 3, 5, false);
        put(&mut board, Chessman::White, 4, 7, false);
        put(&mut board, Chessman::Black, 4, 5, false);
        put(&mut board, Chessman::White, 3, 6, false);
        put(&mut board, Chessman::Black, 7, 8, false);
        put(&mut board, Chessman::White, 6, 9, false);
        put(&mut board, Chessman::Black, 8, 9, true);
        put(&mut board, Chessman::White, 7, 10, true);
    }
}
