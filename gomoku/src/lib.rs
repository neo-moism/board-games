#![allow(dead_code)]
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Chessman {
    Black,
    White,
}

const LINE_COUNT: usize = 15;

pub struct Board {
    states: [[Option<Chessman>; LINE_COUNT]; LINE_COUNT],
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
    fn dump_states(&self) -> Vec<Vec<u8>> {
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

    fn put_piece(&mut self, c: Chessman, x: usize, y: usize) {
        let ok = self.check(c, x, y);
        if !ok {
            return;
        }
        self.states[x][y] = Some(c);
        self.last = c;
        let _ = self.wins();
    }

    fn check(&self, c: Chessman, x: usize, y: usize) -> bool {
        if self.last == c || x >= LINE_COUNT || y >= LINE_COUNT {
            // TODO err
            return false;
        }
        if self.states[x][y].is_some() {
            return false;
        }
        if Chessman::Black == c {
            // TODO check
        }
        false
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
