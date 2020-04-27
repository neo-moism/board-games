#![allow(dead_code)]
use actix::prelude::*;
use gomoku;
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Message)]
#[rtype(result = "()")]
pub struct StrMsg(pub String);

// ------------ GomokuRoom
#[derive(Message)]
#[rtype(result = "Result<bool, gomoku::Error>")]
pub struct PutChessman {
    pub black: bool,
    pub x: usize,
    pub y: usize,
}
pub struct GomokuRoom {
    board: gomoku::Board,
    history: Vec<(gomoku::Chessman, usize, usize)>,
    black: usize,
    white: usize,
}
impl Actor for GomokuRoom {
    type Context = Context<Self>;
}
impl Handler<PutChessman> for GomokuRoom {
    type Result = <PutChessman as Message>::Result;
    fn handle(&mut self, msg: PutChessman, _: &mut Self::Context) -> Self::Result {
        let c = if msg.black {
            gomoku::Chessman::Black
        } else {
            gomoku::Chessman::White
        };
        let r = self.board.put_piece(c, msg.x, msg.y);
        if r.is_ok() {
            self.history.push((c, msg.x, msg.y));
        }
        r
    }
}
// ------------ End of GomokuRoom

pub(crate) struct Hall {
    sessions: HashMap<usize, Recipient<StrMsg>>,
    online_users: HashMap<usize, Arc<User>>,
    gomoku_q: VecDeque<(usize, Instant)>,
    gomoku_queued_users: HashSet<usize>,
    gomoku_rooms: HashMap<usize, Arc<Mutex<GomokuRoom>>>,
}

impl Actor for Hall {
    type Context = Context<Self>;
}

struct User {
    name: String,
    avatar: String,
}

/// New session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<StrMsg>,
    pub name: String,
}

#[derive(Message)]
#[rtype("()")]
pub struct Disconnect(pub usize);

impl Handler<Connect> for Hall {
    type Result = usize;
    fn handle(&mut self, msg: Connect, _: &mut Context<Hall>) -> usize {
        let Connect { addr, name } = msg;
        let id = rand::thread_rng().gen();
        let user = User {
            avatar: name.clone(),
            name,
        };
        let user = Arc::new(user);
        self.online_users.insert(id, user.clone());
        self.sessions.insert(id, addr);
        id
    }
}

impl Handler<Disconnect> for Hall {
    type Result = ();
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Hall>) {
        let Disconnect(id) = msg;
        self.online_users.remove(&id);
        self.sessions.remove(&id);
        self.gomoku_queued_users.remove(&id);
        // TODO send to the room if exists
    }
}

use rand::prelude::*;

impl Hall {
    fn logout(&mut self, player_id: usize) {
        self.online_users.remove(&player_id);
    }

    fn play_gomoku(&mut self, player: &usize) -> Option<Arc<Mutex<GomokuRoom>>> {
        if self.gomoku_queued_users.contains(player) {
            // Waiting for a target
            return None;
        }
        while !self.gomoku_q.is_empty() {
            let (another, _) = self.gomoku_q.pop_front().unwrap();
            // Check if the target canceled
            if self.gomoku_queued_users.remove(&another) {
                // Matched, create a room
                let room = GomokuRoom {
                    board: gomoku::Board::default(),
                    black: another.clone(),
                    white: player.clone(),
                    history: vec![],
                };
                let room = Arc::new(Mutex::new(room));
                self.gomoku_rooms.insert(player.clone(), room.clone());
                self.gomoku_rooms.insert(another, room.clone());
                return Some(room);
            }
        }
        // Just wait for another guy
        self.gomoku_q.push_back((player.clone(), Instant::now()));
        self.gomoku_queued_users.insert(player.clone());
        return None;
    }
}
