#[macro_use]
extern crate failure;
extern crate futures;
extern crate hyper;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate serde_json;
#[macro_use]
extern crate base64_serde;

mod colors;

use base64::STANDARD;
use colors::Color;
use futures::{future, Future, Stream};
use hyper::{service::service_fn, Body, Error, Method, Request, Response, Server, StatusCode};
use rand::distributions::{Bernoulli, Normal, Uniform};
use rand::Rng;
use std::{
    cmp::{max, min},
    ops::Range,
};

base64_serde_type!(Base64Standard, STANDARD);

static INDEX: &[u8] = b"Random service";

fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();
    let builder = Server::bind(&addr);
    let server = builder.serve(|| service_fn(microservice_handler));
    let server = server.map_err(drop);
    hyper::rt::run(server);
}

// RngResponse
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum RngResponse {
    Value(f64),
    #[serde(with = "Base64Standard")]
    Bytes(Vec<u8>),
    Color(Color),
}

// RngRequest
#[derive(Deserialize)]
#[serde(tag = "distribution", content = "parameters", rename_all = "lowercase")]
enum RngRequest {
    Uniform {
        #[serde(flatten)]
        range: Range<i32>,
    },
    Normal {
        mean: f64,
        std_dev: f64,
    },
    Bernoulli {
        p: f64,
    },
    Shuffle {
        #[serde(with = "Base64Standard")]
        data: Vec<u8>,
    },
    Color {
        from: Color,
        to: Color,
    },
}

// Handler
fn microservice_handler(
    req: Request<Body>,
) -> Box<Future<Item = Response<Body>, Error = Error> + Send> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/random") => {
            Box::new(future::ok(Response::new(INDEX.into())))
        }
        (&Method::POST, "/random") => {
            let body = req.into_body().concat2().map(|chunks| {
                let res = serde_json::from_slice::<RngRequest>(chunks.as_ref())
                    .map(handle_request)
                    .and_then(|resp| serde_json::to_string(&resp));

                match res {
                    Ok(body) => Response::new(body.into()),
                    Err(err) => Response::builder()
                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                        .body(err.to_string().into())
                        .unwrap(),
                }
            });
            Box::new(body)
        }
        _ => {
            let resp = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Not Found".into())
                .unwrap();
            Box::new(future::ok(resp))
        }
    }
}

// Color_range
fn color_range(from: u8, to: u8) -> Uniform<u8> {
    let (from, to) = (min(from, to), max(from, to));
    Uniform::new_inclusive(from, to)
}

// Handling requests
fn handle_request(request: RngRequest) -> RngResponse {
    let mut rng = rand::thread_rng();

    match request {
        RngRequest::Uniform { range } => {
            let value = rng.sample(Uniform::from(range)) as f64;
            RngResponse::Value(value)
        }
        RngRequest::Normal { mean, std_dev } => {
            let value = rng.sample(Normal::new(mean, std_dev)) as f64;
            RngResponse::Value(value)
        }
        RngRequest::Bernoulli { p } => {
            let value = rng.sample(Bernoulli::new(p)) as i8 as f64;
            RngResponse::Value(value)
        }
        RngRequest::Shuffle { mut data } => {
            rng.shuffle(&mut data);
            RngResponse::Bytes(data)
        }
        RngRequest::Color { from, to } => {
            let red = rng.sample(color_range(from.red, to.red));
            let green = rng.sample(color_range(from.green, to.green));
            let blue = rng.sample(color_range(from.blue, to.blue));
            RngResponse::Color(Color { red, green, blue })
        }
    }
}
