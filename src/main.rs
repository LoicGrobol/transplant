use std::io::{Write, BufWriter};

use aho_corasick;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 || &args[1] == "-h" || &args[1] == "--help" {
        eprintln!("Usage: transplante <original> <replacement> <source-file> <target-file>");
        std::process::exit(1);
    }
    let original = &args[1].as_bytes();
    let replacement = &args[2].as_bytes();
    let source_file = &args[3];
    let target_file = &args[4];

    let source_reader = std::fs::File::open(source_file).expect("Can't open source file");
    let target_writer = BufWriter::new(std::fs::File::create(target_file).expect("Can't open target file"));
    let mut n_replaced = 0;
    let automaton = aho_corasick::AhoCorasick::new(&[original]);
    automaton.stream_replace_all_with(source_reader, target_writer, |_, _, wtr| {
        n_replaced+=1;
        wtr.write_all(replacement)
    }).unwrap();
    eprintln!("Replaced {} times", n_replaced);
}