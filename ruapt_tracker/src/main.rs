mod data;
mod error;
mod storage;
mod util;

use bendy::serde::{from_bytes, to_bytes};
use data::AnnounceRequestData;
use futures::prelude::*;
use std::sync::Arc;
use storage::redis::DB;
use storage::Storage;
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio_serde::formats::*;
use tokio_serde::{Deserializer, Framed, Serializer};
use tokio_util::codec::{FramedRead, LengthDelimitedCodec, LinesCodec};

async fn tracker_loop(socket: tokio::net::TcpStream, db: std::sync::Arc<storage::redis::DB>) {
    let (reader, mut writer) = socket.into_split();
    let mut lines = FramedRead::new(reader, LinesCodec::new());
    while let Ok(Some(msg)) = lines.try_next().await {
        let a: AnnounceRequestData = match from_bytes(msg.as_bytes()) {
            Ok(a) => a,
            _ => continue,
        };
        println!("{:?}", a);
        let response = db.announce(&a).await.unwrap();
        if let Some(r) = response {
            let bytes = to_bytes(&r).unwrap();
            writer.write_all(&bytes).await;
        }
    }
}

async fn compaction_loop(db: std::sync::Arc<storage::redis::DB>) {
    loop {
        // TODO: make the compaction more smooth
        // TODO: here need some logs
        db.compaction().await;
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("<================Rua PT is running================>");
    let db = Arc::new(DB::new(
        "redis://:1234567890@127.0.0.1:1711/0",
        "redis://:1234567890@127.0.0.1:1711/1",
    ));
    tokio::spawn(compaction_loop(db.clone()));
    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(tracker_loop(socket, db.clone()));
    }
}
