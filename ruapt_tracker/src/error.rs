pub type TrackerResult<T> = Result<T, TrackerError>;

#[derive(Debug)]
pub enum TrackerError {
    RedisError(String),
}

impl From<deadpool_redis::redis::RedisError> for TrackerError {
    fn from(e: deadpool_redis::redis::RedisError) -> Self {
        TrackerError::RedisError(format!("{:?}", e))
    }
}
