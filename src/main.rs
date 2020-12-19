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

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Server};
use object_pool::Pool;
use std::sync::Arc;
use std::{convert::Infallible, net::SocketAddr};
use tokio::task;

mod attempt1;
mod attempt2;
mod attempt3;
mod attempt4;
mod cli;
mod request_generated;

#[macro_use]
extern crate lazy_static;
lazy_static! {
    /*
    static ref POOL: Arc<Pool<flatbuffers::FlatBufferBuilder<'static>>> =
        Arc::new(Pool::new(1000, || {
            flatbuffers::FlatBufferBuilder::new_with_capacity(4096)
        }));
    */

        static ref POOL: Arc<Pool<Box<flatbuffers::FlatBufferBuilder<'static>>>> =
        Arc::new(Pool::new(100000, || {
            Box::new(flatbuffers::FlatBufferBuilder::new_with_capacity(4096))
        }));
}

#[tokio::main]
async fn main() {
    let matches = cli::args();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let mut record = true;
    let mut output_directory = "";
    if let Some(matches) = matches.subcommand_matches("record") {
        record = true;

        output_directory = matches
            .value_of("output")
            .expect("Unexpected! `output` is marked required!");
    }

    // let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    // let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });
    let make_svc = make_service_fn(|_conn| {
        // we must clone the 'tx' to be something owned by the closure
        // the new `tx` will be tied to the scope of the closure and not to
        // caller, `main`. This must be outside out `async` block below.
        // that is it must be done *now*, not in future.
        let (tx, rx) = tokio::sync::mpsc::channel(1000);

        let output_directory = output_directory.to_string();

        task::spawn(async move { attempt4::recorder(output_directory, rx).await });

        // tx is now a separate clone for each instance of http-connection
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                attempt4::handle(req, record, tx.clone())
                //attempt1::handle(req, record, tx.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
