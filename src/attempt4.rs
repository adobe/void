/*
Copyright 2020 Adobe. All rights reserved.
This file is licensed to you under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License. You may obtain a copy
of the License at http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under
the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR REPRESENTATIONS
OF ANY KIND, either express or implied. See the License for the specific language
governing permissions and limitations under the License.
*/

use flate2::write::DeflateEncoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use hyper::body::Buf;
use hyper::{Body, Request, Response};
use lz4;
use std::convert::Infallible;
use std::io::prelude::*;
use std::io::Write;
use tempfile::Builder;
use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt, BufWriter};
use tokio::sync::mpsc::{Receiver, Sender};

#[allow(dead_code)] // when attempt1 is being tried, this won't be used
pub async fn handle(
    req: Request<Body>,
    record: bool,
    mut tx: Sender<Box<flatbuffers::FlatBufferBuilder<'static>>>,
) -> Result<Response<Body>, Infallible> {
    if record {
        // let mut builder = Box::new(flatbuffers::FlatBufferBuilder::new_with_capacity(4096));
        let mut builder = super::POOL
            .try_pull()
            .expect("unable to get item from pool");
        let (_, mut builder) = builder.detach();
        builder.reset();

        let id = builder.create_string("");
        let method = builder.create_string(req.method().as_str());
        let uri = builder.create_string(&req.uri().to_string());

        // figure out how to read raw header bytes
        let headers = builder.create_string("");

        let body = hyper::body::to_bytes::<Body>(req.into_body())
            .await
            .expect("Reading body failed");

        println!("Received body >{:?}<", body.bytes());

        let body = builder.create_vector::<u8>(body.bytes());

        let buf = super::request_generated::fbr::Request::create(
            &mut builder,
            &super::request_generated::fbr::RequestArgs {
                id: Some(id),
                method: Some(method),
                body: Some(body),
                headers: Some(headers),
                uri: Some(uri),
            },
        );

        builder.finish(buf, None);
        // let finished_bytes_vec = builder.finished_data().to_vec().clone();
        tx.send(builder).await.expect("Unable to send to channel");
    }
    let resp_message = "OK\n";
    Ok(Response::new(resp_message.into()))
}

#[allow(dead_code)] // when attempt1 is being tried, this won't be used
pub async fn recorder(
    output_directory: String,
    mut rx: Receiver<Box<flatbuffers::FlatBufferBuilder<'static>>>,
) {
    println!("Starting recorder");

    let file = Builder::new()
        .prefix("requests_")
        .suffix(".data.gz")
        .rand_bytes(5)
        .tempfile_in(output_directory)
        .expect("Unable to create temp file");

    let filename = file.path().to_string_lossy().to_string();
    file.close();

    let mut raw_fp1 = std::fs::File::create(filename.clone()).expect("Create failed");

    //let mut raw_fp2 = tokio::fs::File::create(filename.clone())
    //    .await
    //    .expect("Create failed");

    let mut encoder = GzEncoder::new(raw_fp1, Compression::Default);

    let mut total_received: i32 = 0;
    let mut total_size = 0;

    println!("Saving requests to {} ...", filename);

    while let Some(builder) = rx.recv().await {
        let finished_data = builder.finished_data();
        println!("About to write >{:?}<", finished_data);
        async {
            encoder.write(finished_data).expect("write failed");
        }
        .await;

        total_received += 1;
        total_size += finished_data.len();
        if total_received % 10000 == 0 {
            println!("Saved {} requests to {}", total_received, filename);
        }
        /* doesn't work - we can't expect size to be rounded number
        if total_size % 10000000 == 0 {
            println!("Saved {} bytes", total_size);
        }
        */
        super::POOL.attach(builder) // return builder to the pool
    }

    println!("Recorder thread finished");
    encoder.flush().expect("Unable to flush to disk")
}
