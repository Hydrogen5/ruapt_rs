use crate::data::*;
use crate::error::*;
use crate::storage::Storage;
use crate::util::get_timestamp;
use async_trait::async_trait;
use deadpool::managed;
use deadpool_redis::{
    redis::{AsyncCommands, AsyncIter, RedisError},
    Config, ConnectionWrapper, Pipeline, PoolError,
};

type Conection = managed::Object<ConnectionWrapper, RedisError>;
type Pool = managed::Pool<ConnectionWrapper, RedisError>;

pub struct DB {
    torrent_pool: Pool,
    user_pool: Pool,
}

impl DB {
    /// The uri format is `redis://[<username>][:<passwd>@]<hostname>[:port]`  
    ///
    /// And we will take db 1 to store torrent connect info and db 2 to store
    /// the info of users.
    pub fn new(torrent_uri: &str, user_uri: &str) -> Self {
        let mut cfg = Config::default();
        assert!(torrent_uri != user_uri);
        cfg.url = Some(torrent_uri.to_string());
        let torrent_pool = cfg.create_pool().expect("Create Redis Pool Failed!");
        cfg.url = Some(user_uri.to_string());
        let user_pool = cfg.create_pool().expect("Create Redis Pool Failed!");
        DB {
            torrent_pool,
            user_pool,
        }
    }
}

impl DB {
    async fn get_torrent_con_with_delay(&self) -> TrackerResult<Conection> {
        loop {
            match self.torrent_pool.try_get().await {
                Ok(con) => break Ok(con),
                Err(PoolError::Timeout(_)) => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    continue;
                }
                _ => return Err(TrackerError::RedisError("Pool error".into())),
            }
        }
    }
    async fn get_torrent_con_no_delay(&self) -> TrackerResult<Conection> {
        loop {
            match self.torrent_pool.get().await {
                Ok(con) => break Ok(con),
                Err(PoolError::Timeout(_)) => continue,
                _ => return Err(TrackerError::RedisError("Pool error".into())),
            }
        }
    }
    async fn get_user_con_no_delay(&self) -> TrackerResult<Conection> {
        loop {
            match self.user_pool.get().await {
                Ok(con) => break Ok(con),
                Err(PoolError::Timeout(_)) => continue,
                _ => return Err(TrackerError::RedisError("Pool error".into())),
            }
        }
    }
}

#[async_trait]
impl Storage for DB {
    async fn compaction(&self) -> TrackerResult<()> {
        let mut con1 = self.get_torrent_con_with_delay().await?;
        // fuck borrow check
        // cannot reuse con1 because the cursor take the mut borrow
        // TODO: maybe rewrite it with raw redis command
        let mut con2 = self.get_torrent_con_with_delay().await?;
        let mut cursor: AsyncIter<String> = con1.scan().await?;
        let mut p = Pipeline::with_capacity(10);
        let t = get_timestamp() - 300;
        let mut cnt = 0;
        while let Some(key) = cursor.next_item().await {
            p.cmd("ZREMRANGEBYSCORE").arg(key).arg(t).ignore();
            cnt += 1;
            if cnt % 10 == 0 {
                p.execute_async(&mut con2).await?;
                p.clear();
            }
        }
        p.execute_async(&mut con2).await?;
        Ok(())
    }

    async fn scrape(&self) -> TrackerResult<()> {
        todo! {}
    }

    async fn announce(
        &self,
        data: &AnnounceRequestData,
    ) -> TrackerResult<Option<AnnounceResponseData>> {
        use crate::data::Action::*;
        // do nothing, the compaction will remove it
        // in few minutes.
        // maybe remove it from the user?
        let mut user_con = self.get_user_con_no_delay().await?;
        let t_id = format!("to_{}", data.torrent_id);
        if let Stopped = data.action {
            user_con.srem(&data.peer_id, &t_id).await?;
            return Ok(None);
        }
        // use t_id instead info_hash to decrease memory usage
        // actually, if it is worth using t_id is unknown
        // then get the return value
        let mut p = Pipeline::with_capacity(4);
        p.sadd(&data.peer_id, &t_id);
        p.expire(&data.peer_id, 300);
        p.execute_async(&mut user_con).await?;
        p.clear();
        let now = get_timestamp();
        p.zadd(&t_id, data.encode_info(), now);
        p.expire(&t_id, 300);
        let mut to_con = self.get_torrent_con_no_delay().await?;
        p.execute_async(&mut to_con).await?;
        // ZRANGEBYSCORE t_id now-300 +inf LIMIT 0 num_want
        let ret: Vec<String> = to_con
            .zrangebyscore_limit(&t_id, now - 300, "+inf", 0, data.num_want)
            .await?;
        // println!("{:?}", ret);
        // todo! {}
        Ok(Some(AnnounceResponseData {
            peers: ret.iter().map(Peer::from).collect(),
        }))
    }
}
