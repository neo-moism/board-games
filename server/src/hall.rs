#![allow(dead_code)]
use gomoku;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

pub trait HallStorage {
    fn get_avatar(&self, username: &String) -> String;
}

struct MemStorage {}
impl HallStorage for MemStorage {
    fn get_avatar(&self, username: &String) -> String {
        username.clone()
    }
}

struct GomokuRoom {
    board: gomoku::Board,
    history: Vec<(gomoku::Chessman, usize, usize)>,
    black: String,
    white: String,
}

pub(crate) struct Hall<S: HallStorage> {
    storage: S,
    online_users: HashMap<String, Arc<User>>,
    gomuku_q: VecDeque<(String, Instant)>,
    queued_users: HashSet<String>,
    gomuku_rooms: HashMap<String, Arc<Mutex<GomokuRoom>>>,
}

struct User {
    name: String,
    avatar: String,
}

impl<S: HallStorage> Hall<S> {
    fn login(&mut self, username: String, _password: &str) -> Arc<User> {
        let avatar = self.storage.get_avatar(&username);
        let user = User {
            name: username,
            avatar,
        };
        let user = Arc::new(user);
        self.online_users.insert(user.name.clone(), user.clone());
        user
    }

    fn logout(&mut self, username: &String) {
        self.online_users.remove(username);
    }

    fn play_gomoku(&mut self, player: &String) -> Option<Arc<Mutex<GomokuRoom>>> {
        if self.queued_users.contains(player) {
            // Waiting for a target
            return None;
        }
        while !self.gomuku_q.is_empty() {
            let (another, _) = self.gomuku_q.pop_front().unwrap();
            // Check if the target canceled
            if self.queued_users.remove(&another) {
                // Matched, create a room
                let room = GomokuRoom {
                    board: gomoku::Board::default(),
                    black: another.clone(),
                    white: player.clone(),
                    history: vec![],
                };
                let room = Arc::new(Mutex::new(room));
                self.gomuku_rooms.insert(player.clone(), room.clone());
                self.gomuku_rooms.insert(another, room.clone());
                return Some(room);
            }
        }
        // Just wait for another guy
        self.gomuku_q.push_back((player.clone(), Instant::now()));
        self.queued_users.insert(player.clone());
        return None;
    }
}
