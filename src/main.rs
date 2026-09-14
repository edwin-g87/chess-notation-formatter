use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process::ExitCode;

mod notation;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let input = match args.as_slice() {
        [] => read_stdin(),
        [path] if path == "-" => read_stdin(),
        [path] => fs::read_to_string(path).map_err(|e| format!("{path}: {e}")),
        _ => {
            eprintln!("usage: chessfmt [FILE]");
            eprintln!("reads from FILE, or from stdin if FILE is omitted or \"-\"");
            return ExitCode::from(2);
        }
    };

    let input = match input {
        Ok(text) => text,
        Err(err) => {
            eprintln!("chessfmt: {err}");
            return ExitCode::FAILURE;
        }
    };

    let output = notation::normalize(&input);

    if io::stdout().write_all(output.as_bytes()).is_err() {
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn read_stdin() -> Result<String, String> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| e.to_string())?;
    Ok(buf)
}
