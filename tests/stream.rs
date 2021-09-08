#!/usr/bin/env cargo-play
//# tokio = { version = "*", features = ["rt-multi-thread", "macros", "sync"] }
use std::{sync::Arc, time::Instant};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        // chunk debug 140ms release 40ms
        let mut i = 0;
        while i < 100000 {
            let n = 10;
            tx.send(
                (0..n)
                    .map(|j| Arc::new((i + j).to_string().repeat(100)))
                    .collect::<Vec<_>>()
            )
            .unwrap();
            i += n;
        }
        // item debug 1.9s release 810ms
        // let mut i = 0;
        // while i < 1000000 {
        //    let n = 10;
        //    tx.send(Arc::new(i.to_string().repeat(100))).unwrap();
        //    i += 1;
        //}
    });
    let handle = tokio::spawn(async move {
        let start = Instant::now();
        while let Some(x) = rx.recv().await {
            for _ in x {}
        }
        println!("{:?}", Instant::now() - start);
    });
    handle.await.unwrap();
}
