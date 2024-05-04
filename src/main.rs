use std::collections::BinaryHeap;
use std::fs::File;

use std::io::BufReader;

mod args;
mod date_utils;
mod log_seek;

fn main() {
    let args = args::cli_args();
    let mut heap = get_heap_of_files(&args);

    while !heap.is_empty() {
        let mut next = match heap.pop() {
            Some(seekable) => {
                if let Some(ref next) = seekable.next {
                    println!("{}", next.value);
                }

                seekable
            }
            None => break,
        };

        if !next.advance() {
            continue;
        }

        if let Some(next) = is_valid(next, &args) {
            heap.push(next);
        }
    }
}

fn is_valid(seekable: log_seek::Seekable, cli: &args::Args) -> Option<log_seek::Seekable> {
    if let Some(ref start) = cli.start {
        if seekable.last.sort < *start {
            return None;
        }
    }

    if let Some(ref end) = cli.end {
        if seekable.last.sort > *end {
            return None;
        }
    }

    Some(seekable)
}

fn get_heap_of_files(cli: &args::Args) -> BinaryHeap<log_seek::Seekable> {
    let mut heap = BinaryHeap::new();

    for file in &cli.files {
        let file = File::open(file).unwrap();
        let len = file.metadata().unwrap().len();
        let reader = BufReader::new(file);

        match log_seek::Seekable::new(reader, len, cli.start) {
            Some(seekable) => {
                if let Some(seekable) = is_valid(seekable, cli) {
                    heap.push(seekable);
                }
            }
            None => {
                println!("Seekable not found");
            }
        }
    }

    heap
}
