use std::io::{self, Read};

extern crate mmpp;

fn main() {
    let mut buffer = String::new();
    let _ = io::stdin().read_to_string(&mut buffer);
    match mmpp::parse_metric(buffer.as_ref()) {
        Ok(metric) => println!("{}", mmpp::pretty_print(metric)),
        Err(err) => {
            println!("{}", err);
            std::process::exit(1)
        }
    }
}
