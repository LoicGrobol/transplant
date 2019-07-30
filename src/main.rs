use memchr;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 || &args[1] == "-h" || &args[1] == "--help" {
        eprintln!("Usage: transplante <original> <replacement> <source-file> <target-file>");
        std::process::exit(1);
    }
    let original = &args[1];
    let replacement = &args[2];
    let source_file = &args[3];
    let target_file = &args[4];

    let mut source_reader =
        std::io::BufReader::new(std::fs::File::open(source_file).expect("Can't open source file"));
    let mut target_writer = std::io::BufWriter::new(
        std::fs::File::create(target_file).expect("Can't open target file"),
    );

    let n_replaced = replace(
        original,
        replacement,
        &mut source_reader,
        &mut target_writer,
    )
    .unwrap();
    eprintln!("Replaced {} times", n_replaced);
}

fn replace<R: std::io::BufRead, W: std::io::Write>(
    original: &str,
    replacement: &str,
    source: &mut R,
    target: &mut W,
) -> Result<i64, std::io::Error> {
    let original_bytes = original.as_bytes();
    let backtracking_points = find_backtracking_points(&original_bytes);
    let first_byte = original_bytes[0];
    let replacement_bytes = replacement.as_bytes();
    let mut n_replaced = 0;
    loop {
        // Not completely idiomatic I think but more readable
        let found = advance_to(first_byte, source, target)?;
        if !found {
            break;
        };
        let mut validated = 0;
        while validated < original_bytes.len() {
            let buf_content = match source.fill_buf() {
                Ok(x) => x,
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            let (sync, used) = {
                let mut sync = true;
                let mut buf_pos = 0;
                while buf_pos < buf_content.len() {
                    if buf_content[buf_pos] != original_bytes[validated] {
                        sync = false;
                        break;
                    }
                    buf_pos += 1;
                    validated += 1;
                    if validated == original_bytes.len() {
                        break;
                    };
                }
                (sync, buf_pos)
            };
            source.consume(used);
            if !sync {
                let backtrack = backtracking_points
                    .iter()
                    .take_while(|(i, _)| i < &validated)
                    .find(|(_, non_return)| non_return < &validated);
                match backtrack {
                    Some((i, _)) => validated -= i,
                    None => break,
                };
            }
        }

        if validated == original_bytes.len() {
            target.write(replacement_bytes)?;
            n_replaced += 1;
        } else {
            target.write(&original_bytes[..validated])?;
        }
    }
    return Ok(n_replaced);
}

fn advance_to<R: std::io::BufRead, W: std::io::Write>(
    needle: u8,
    input: &mut R,
    output: &mut W,
) -> Result<bool, std::io::Error> {
    loop {
        let current = match input.fill_buf() {
            Ok(n) => n,
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        if current.len() == 0 {
            return Ok(false);
        }
        let (found, used) = match memchr::memchr(needle, current) {
            Some(i) => (true, i),
            None => (false, current.len()),
        };
        match output.write(&current[..used]) {
            Ok(n) => {
                if n < used {
                    panic!("Can't write to target")
                }
            }
            Err(e) => return Err(e),
        };
        input.consume(used);
        if found {
            break;
        }
    }
    return Ok(true);
}

fn find_backtracking_points(sequence: &[u8]) -> Vec<(usize, usize)> {
    let first_byte = sequence[0];
    let mut res: Vec<(usize, usize)> = vec![];
    let mut last_match = 0;
    // Precompute backtracking conditions
    while let Some(i) = memchr::memchr(first_byte, &sequence[last_match + 1..]) {
        // COMBAK: probably not optimal
        let mut repeat_len = 1;
        let repeat_start = last_match + 1 + i;
        while sequence[repeat_len] == sequence[repeat_start + repeat_len] {
            repeat_len += 1;
        }
        res.push((repeat_start, repeat_start + repeat_len));
        last_match = repeat_start;
    }
    res
}
