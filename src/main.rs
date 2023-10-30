use std::fs::{File, OpenOptions};
use std::io::Write;
use std::{fs, io};

use clap::Parser;
use indexmap::IndexSet;

#[derive(Parser, Debug)]
#[command(author = "wese", version, about = "simple tool to write non - duplicate lines to a file", long_about = None)]
struct Cli {
    #[arg(short, long, help = "do not output new lines to stdout")]
    quiet_mode: bool,

    #[arg(short, long, help = "sort lines (natsort)")]
    sort: bool,

    #[arg(short, long, help = "trim whitespaces")]
    trim: bool,

    #[arg(
        short,
        long,
        help = "rewrite existing destination file to remove duplicates"
    )]
    rewrite: bool,

    #[arg(long, help = "do not write to file, only output what would be written")]
    dry_run: bool,

    #[arg(help = "destination file")]
    filepath: String,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();
    let filepath = &args.filepath;

    let mut lines = match load_file(&args) {
        Ok(value) => value,
        Err(value) => Err(value)?,
    };

    if !args.dry_run && args.rewrite {
        let mut f = OpenOptions::new()
            .append(false)
            .truncate(true)
            .write(true)
            .create(true)
            .open(filepath)
            .expect("failed to open file");

        for line in lines.iter() {
            writeln!(f, "{}", line).expect("failed to write to file");
        }

        drop(f);
    }

    let stdin_lines = io::stdin().lines();
    for stdin_line in stdin_lines {
        let stdin_line = stdin_line?;

        if !is_newline(&args, &lines, &stdin_line) {
            continue;
        }

        lines.insert(stdin_line.to_string());

        if !args.quiet_mode {
            println!("{}", stdin_line);
        }

        if !args.dry_run {
            let mut f: Option<File> = None;

            if f.is_none() {
                f = Some(
                    OpenOptions::new()
                        .append(true)
                        .write(true)
                        .create(true)
                        .open(filepath)
                        .expect("failed to open file"),
                );
            }

            let mut file = f.unwrap();
            writeln!(file, "{}", stdin_line).expect("failed to write to file");

            drop(file);
        }
    }

    if args.sort && !args.dry_run {
        let mut f = OpenOptions::new()
            .append(false)
            .write(true)
            .create(false)
            .open(&args.filepath)
            .expect("failed to open file");

        lines.sort_by(|a, b| natord::compare_ignore_case(a.as_str(), b.as_str()));

        for line in lines.iter() {
            writeln!(f, "{}", line)?;
        }

        drop(f);
    }

    Ok(())
}

fn load_file(args: &Cli) -> Result<IndexSet<String>, io::Error> {
    let mut lines = IndexSet::new();
    match fs::read_to_string(&args.filepath) {
        Ok(data) => {
            for line in data.lines() {
                if !is_newline(args, &lines, &line) {
                    continue;
                }

                lines.insert(line.to_string());
            }
        }
        Err(err) if err.kind() != io::ErrorKind::NotFound => {
            eprintln!("failed to open _file for reading: {}", err);

            return Err(err);
        }
        _ => {}
    }

    Ok(lines)
}

fn is_newline(args: &Cli, lines: &IndexSet<String>, line: &str) -> bool {
    let line = if args.trim { line.trim() } else { line };
    if line == "" {
        return false;
    }

    if lines.contains(line) {
        return false;
    }

    return true;
}
