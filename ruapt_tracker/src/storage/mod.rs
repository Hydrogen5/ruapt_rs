pub mod redis;
// pub mod memory;

use crate::data::*;
use crate::error::TrackerResult;
use async_trait::async_trait;

#[async_trait]
pub trait Storage {
    async fn compaction(&self) -> TrackerResult<()>;
    async fn scrape(&self) -> TrackerResult<()>;
    async fn announce(
        &self,
        data: &AnnounceRequestData,
    ) -> TrackerResult<Option<AnnounceResponseData>>;
}
