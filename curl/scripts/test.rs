#!/usr/bin/env run-cargo-script

//! ```cargo
//! [package]
//! edition = "2018"
//! [dependencies]
//! anyhow = "1.0.32"
//! futures = "0.3.5"
//! qiniu-curl = { version = "*", path = "../", features = ["async"] }
//! ```

use futures::{executor::block_on, future::try_join_all};
use qiniu_curl::{CurlHTTPCaller, HTTPCaller, Request};

fn main() -> anyhow::Result<()> {
    let url = "https://www.baidu.com";
    let caller = CurlHTTPCaller::default();

    println!("starting 1 ...");
    let futures = (0..100).map(|i| async {
        println!(">>");
        let future = caller
            .async_call(&Request::builder().url(url).build())
            .await;
        println!("<<");
        future
    });
    println!("starting 2 ...");

    block_on(async { try_join_all(futures).await })?;
    Ok(())
}
