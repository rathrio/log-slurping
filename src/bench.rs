use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufReader};

const LARGE_FILE: &str = "./logs/large-log.log";

// ~ 1.060 s
fn buffered_with_capacity() {
    let file = File::open(LARGE_FILE).unwrap();
    let reader = BufReader::with_capacity(128 * 1024, file);

    reader.lines().for_each(|line| {
        println!("{}", line.unwrap());
    })
}

// ~ 1.068 s
fn buffered() {
    let file = File::open(LARGE_FILE).unwrap();
    let reader = BufReader::new(file);

    reader.lines().for_each(|line| {
        println!("{}", line.unwrap());
    })
}

// ~ 1.065 s
fn everything_at_once() {
    let data = fs::read_to_string(LARGE_FILE).expect("Unable to read file");
    data.lines().for_each(|line| {
        println!("{}", line);
    })
}

// In conclusion: doesn't matter when one's only parsing one large file at a time.
