#![allow(dead_code)]
use crate::handler::Resp;
use actix::prelude::*;
use gomoku;
use rand::prelude::*;
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

pub struct GomokuRoom {
    board: gomoku::Board,
    history: Vec<(gomoku::Chessman, usize, usize)>,
    black: usize,
    white: usize,
    black_ready: bool,
    whilte_ready: bool,
    black_r: Recipient<Resp>,
    white_r: Recipient<Resp>,
}

pub enum GomokuState {
    Waiting,
    NotReady,
    Playing,
}
impl Actor for GomokuRoom {
    type Context = Context<Self>;
}

use crate::handler;
impl Handler<GomokuMsg> for GomokuRoom {
    type Result = <GomokuMsg as Message>::Result;
    fn handle(&mut self, msg: GomokuMsg, ctx: &mut Self::Context) {
        match msg {
            GomokuMsg::Ready(id) => {
                if self.black == id {
                    self.black_ready = true;
                }
                if self.white == id {
                    self.whilte_ready = true;
                }
                if self.whilte_ready && self.black_ready {
                    let _ = self
                        .black_r
                        .do_send(handler::Resp::GomokuStart(ctx.address()));
                    let _ = self
                        .white_r
                        .do_send(handler::Resp::GomokuStart(ctx.address()));
                } else {
                    if self.black == id {
                        let _ = self.black_r.do_send(Resp::Str("not_ready".to_string()));
                    } else {
                        let _ = self.white_r.do_send(Resp::Str("not_ready".to_string()));
                    }
                }
            }
            GomokuMsg::Put(id, x, y) => {
                let c = if self.black == id {
                    gomoku::Chessman::Black
                } else {
                    gomoku::Chessman::White
                };
                let r = self.board.put_piece(c, x, y);
                if r.is_ok() {
                    self.history.push((c, x, y));
                }
                // TODO send result to players
            }
            GomokuMsg::Quit(player) => {
                if self.black == player {
                    self.black = 0;
                }
                if self.white == player {
                    self.white = 0;
                }
                // TODO Check if game finished, send result
                if self.black == 0 && self.white == 0 {
                    ctx.stop();
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Hall {
    sessions: HashMap<usize, Recipient<Resp>>,
    online_users: HashMap<usize, Arc<User>>,
    gomoku_q: VecDeque<(usize, Instant)>,
    gomoku_queued_users: HashSet<usize>,
    gomoku_rooms: HashMap<usize, Addr<GomokuRoom>>,
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
    pub addr: Recipient<Resp>,
    pub name: String,
}

// TODO
#[derive(Message)]
#[rtype("()")]
pub enum HallMsg {
    StartGomoku(usize),
    CancelGomoku(usize),
    Chat(ChatMsg),
}

#[derive(Message)]
#[rtype("()")]
pub enum GomokuMsg {
    Ready(usize),
    Put(usize, usize, usize),
    Quit(usize),
}

pub struct ChatMsg {
    pub content: String,
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
        if let Some(_addr) = self.gomoku_rooms.get(&id) {
            // TODO addr.do_send();
        }
        self.gomoku_rooms.remove(&id);
    }
}

impl Handler<HallMsg> for Hall {
    type Result = ();
    fn handle(&mut self, msg: HallMsg, _: &mut Context<Hall>) {
        match msg {
            // Broadcast
            HallMsg::Chat(msg) => {
                let ChatMsg { content, mut name } = msg;
                name.push(':');
                name.push(' ');
                name.push_str(&content);
                for s in self.sessions.values() {
                    let _ = s.do_send(Resp::Str(name.clone()));
                }
            }
            HallMsg::StartGomoku(player) => {
                self.start_gomoku(player);
            }
            HallMsg::CancelGomoku(player) => {
                self.gomoku_queued_users.remove(&player);
                let _ = self
                    .sessions
                    .get(&player)
                    .unwrap()
                    .do_send(Resp::Str("TODO canceled".to_owned()));
            }
        };
    }
}

impl Hall {
    pub fn start_gomoku(&mut self, player: usize) {
        if self.gomoku_rooms.contains_key(&player) {
            // TODO room invalid?
            let _ = self
                .sessions
                .get(&player)
                .unwrap()
                .do_send(Resp::Str("TODO playing".to_owned()));
            return;
        }
        if self.gomoku_queued_users.contains(&player) {
            let _ = self
                .sessions
                .get(&player)
                .unwrap()
                .do_send(Resp::Str("TODO waiting".to_owned()));
            return;
        }
        while !self.gomoku_q.is_empty() {
            let (another, _) = self.gomoku_q.pop_front().unwrap();
            // Check if the target canceled
            if self.gomoku_queued_users.remove(&another) {
                // Matched, create a room
                let room = GomokuRoom {
                    board: gomoku::Board::default(),
                    black: another,
                    white: player,
                    history: vec![],
                    whilte_ready: false,
                    black_ready: false,
                    // TODO
                    black_r: self.sessions.get(&another).unwrap().clone(),
                    white_r: self.sessions.get(&player).unwrap().clone(),
                };
                let addr = room.start();
                self.gomoku_rooms.insert(player.clone(), addr.clone());
                self.gomoku_rooms.insert(another, addr);
                let _ = self
                    .sessions
                    .get(&player)
                    .unwrap()
                    .do_send(Resp::Str("TODO not_ready".to_owned()));
                return;
            }
        }
        self.gomoku_q.push_back((player.clone(), Instant::now()));
        self.gomoku_queued_users.insert(player.clone());
    }
}
