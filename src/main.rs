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
use std::{convert::Infallible, net::SocketAddr};
use tokio::task;
use std::sync::Arc;
use object_pool::Pool;

mod request_generated;
mod attempt1;
mod attempt2;

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
        Arc::new(Pool::new(1000, || {
            Box::new(flatbuffers::FlatBufferBuilder::new_with_capacity(4096))
        }));
    
}


#[tokio::main]
async fn main() {
    let (tx, rx) = crossbeam::channel::bounded(100);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    for i in 0..5 {
        let rx = rx.clone();
        let file_name = format!("foo{}.data", i);
        // task::spawn_blocking(move || attempt2::recorder(file_name, rx));
        task::spawn_blocking(move || attempt1::recorder(file_name, rx));
    }

    // let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    // let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });
    let make_svc = make_service_fn(|_conn| {
        // we must clone the 'tx' to be something owned by the closure
        // the new `tx` will be tied to the scope of the closure and not to
        // caller, `main`. This must be outside out `async` block below.
        // that is it must be done *now*, not in future.
        let tx = tx.clone();
        // tx is now a separate clone for each instance of http-connection
        //let pool: Arc<Pool<Box<flatbuffers::FlatBufferBuilder<'_>>>> =
        //    Arc::new(Pool::new(100, || {
        //        Box::new(flatbuffers::FlatBufferBuilder::new_with_capacity(4096))
        //    }));
        async /* move */ { // move keyword seems optional here - find out why

            // move keyword is very much required in the closure below
            // this function is called for each request. Needs a separate tx clone.
            //
            // `move` keywords moves `tx` to inside closure. without it, 
            // subsequent clones can't be made out of a reference that has disappeared
            //
            // Still a bit confused, but this is all I know at this point.
            // `move` is required here, but why wasn't it required
            // at ..... make_service_fn(|_conn|... closure..above
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {

                // attempt2::handle(req, tx.clone())
                attempt1::handle(req, tx.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
