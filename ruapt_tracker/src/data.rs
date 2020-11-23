use crate::error::*;
use serde::{Deserialize, Serialize};
pub use Action::*;

#[derive(Deserialize, Debug)]
pub struct AnnounceRequestData {
    pub info_hash: String,
    pub peer_id: String,
    pub torrent_id: u64,
    pub ip: String,
    pub port: i32,
    pub action: Action,
    pub num_want: isize,
}

impl AnnounceRequestData {
    // pub fn new() -> Self {
    //     AnnounceRequestData {
    //         info_hash: String::from("ABCDEFG"),
    //         peer_id: String::from("ASSSSSSS"),
    //         torrent_id: 12,
    //         addr: String::from("ASSSSSDASC"),
    //         action: Action::Started,
    //         num_want: 100,
    //     }
    // }
    pub fn encode_info(&self) -> String {
        format!("{}@{}@{}", self.peer_id, self.ip, self.port)
    }
}

#[derive(Deserialize, Debug)]
pub enum Action {
    Completed,
    Started,
    Stopped,
}

#[derive(Serialize, Debug)]
pub struct Peer {
    peer_id: Vec<u8>,
    ip: Vec<u8>,
    port: i32,
}

impl Peer {
    pub fn from(info: &Vec<u8>) -> TrackerResult<Peer> {
        let tmp: Vec<&[u8]> = info.split(|&ch| ch as char == '@').collect();
        if let Some(p_sli) = tmp.get(2) {
            if let Ok(ps) = std::str::from_utf8(p_sli) {
                if let Ok(port) = ps.parse() {
                    return Ok(Peer {
                        peer_id: tmp[0].into(),
                        ip: tmp[1].into(),
                        port,
                    });
                }
            }
        }
        Err(TrackerError::ParseError("Can not convert to Peer"))
    }
}

#[derive(Serialize, Debug)]
pub struct AnnounceResponseData {
    pub peers: Vec<Peer>,
}
