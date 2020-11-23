use crate::data::*;
use crate::error::*;
use crate::storage::Storage;
use crate::util::get_timestamp;
use async_trait::async_trait;
use std::collections::{BTreeMap, HashMap, HashSet};
use tokio::sync::RwLock;

pub struct DB {
    // be pessimistic with its perf...
    user_info: RwLock<HashMap<String, RwLock<HashSet<u64>>>>,
    torrent_info: RwLock<HashMap<String, RwLock<BTreeMap<u64, String>>>>,
}

impl DB {
    /// The uri format is `redis://[<username>][:<passwd>@]<hostname>[:port]`  
    /// And we will take db 1 to store torrent connect info and db 2 to store
    /// the info of users.
    pub fn new() -> Self {
        DB {
            user_info: RwLock::new(HashMap::new()),
            torrent_info: RwLock::new(HashMap::new()),
        }
    }
}

impl DB {}

#[async_trait]
impl Storage for DB {
    async fn compaction(&self) -> TrackerResult<()> {
        todo!()
    }

    async fn scrape(&self) -> TrackerResult<()> {
        todo! {}
    }

    async fn announce(
        &self,
        data: &AnnounceRequestData,
    ) -> TrackerResult<Option<AnnounceResponseData>> {
        if let Stopped = data.action {
            if let Some(hm) = self.user_info.read().await.get(&data.peer_id){
                hm.write().await.remove(&data.torrent_id);
            }
            return Ok(None);
        }
        // let user_map = self.user_info.
        todo!()
    }
}
