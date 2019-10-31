extern crate futures;
extern crate hyper;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

fn main() {
    println!("Hello, world!");
}

// RngResponse
#[derive(Serialize)]
struct RngResponse {
    value: f64,
}

// RngRequest
#[derive(Deserialize)]
#[serde(tag = "distribution", content = "parameters", rename_all = "lowercase")]
enum RngRequest {
    Uniform { range: Range<i32> },
    Normal { mean: f64, std_dev: f64 },
    Bernoulli { p: f64 },
}
