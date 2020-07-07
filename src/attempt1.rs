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

use crossbeam::channel::{Receiver, Sender};
use hyper::{Body, Request, Response};
use hyper::body::Buf;
use lz4;
use std::io::Write;
use std::convert::Infallible;

#[allow(dead_code)] // when attempt2 is being tried, this won't be used
pub async fn handle(
    req: Request<Body>,
    tx: Sender<Vec<u8>>,
) -> Result<Response<Body>, Infallible> {
    let mut builder = flatbuffers::FlatBufferBuilder::new_with_capacity(4096);

    let id = builder.create_string("");
    let method = builder.create_string(req.method().as_str());
    let uri = builder.create_string(&req.uri().to_string());

    // figure out how to read raw header bytes
    let headers = builder.create_string("");

    let body = hyper::body::to_bytes::<Body>(req.into_body())
        .await
        .expect("Reading body failed");

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
    let finished_bytes_vec = builder.finished_data().to_vec().clone();
    tx.send(finished_bytes_vec)
        .expect("unable to write to channel");

    let resp_message = "OK\n";
    Ok(Response::new(resp_message.into()))
}

#[allow(dead_code)] // when attempt2 is being tried, this won't be used
pub fn recorder(file_name: String, rx: Receiver<Vec<u8>>) {
    println!("Starting recorder");

    // let mut file = std::fs::File::create(file_name).expect("Unable to create file");

    let file = std::fs::File::create(file_name).expect("Unable to create file");

    let mut encoder = lz4::EncoderBuilder::new()
        .level(0)
        .build(file)
        .expect("Unable to init lz4");

    let mut total_received: i32 = 0;
    let mut total_size = 0;

    while let Ok(finished_data) = rx.recv() {
        encoder
            .write(finished_data.as_slice())
            .expect("write failed");

        total_received += 1;
        total_size += finished_data.len();
        if total_received % 100000 == 0 {
            println!("Saved {} requests", total_received);
        }
        if total_size % 10000000 == 0 {
            println!("Saved {} bytes", total_size);
        }
    }

    println!("Recorder thread finished");
}
