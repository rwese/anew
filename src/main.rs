use std::{fs, io};
use std::fs::OpenOptions;
use std::io::{Read, Write};

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

    #[arg(short, long, help = "rewrite existing destination file to remove duplicates")]
    rewrite: bool,

    #[arg(long, help = "do not write to file, only output what would be written")]
    dry_run: bool,

    #[arg(help = "destination file")]
    path: String,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    let mut lines = IndexSet::new();
    let _file = fs::File::open(&args.path).is_ok();
    match fs::read_to_string(&args.path) {
        Ok(data) => {
            for line in data.lines() {
                let line = if args.trim { line.trim() } else { line };
                if line == "" {
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

    if !args.dry_run {
        let mut f = OpenOptions::new()
            .append(!args.rewrite)
            .truncate(args.rewrite)
            .write(true)
            .create(true)
            .open(&args.path)
            .expect("failed to open file");

        if args.rewrite {
            for line in lines.iter() {
                writeln!(f, "{}", line).expect("failed to write to file");
            }
        }

        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;

        for line in buffer.lines() {
            let line = if args.trim { line.trim() } else { line };
            if line == "" {
                continue;
            }

            if lines.contains(line) {
                continue;
            }

            lines.insert(line.to_string());

            if !args.quiet_mode {
                println!("{}", line);
            }

            writeln!(f, "{}", line).expect("failed to write to file");
        }
    }

    if args.sort {
        let mut f = OpenOptions::new()
            .append(false)
            .write(true)
            .create(false)
            .open(&args.path)
            .expect("failed to open file");

        lines.sort_by(|a, b| natord::compare(a.as_str(), b.as_str()));

        for line in lines.iter() {
            writeln!(f, "{}", line)?;
        }
    }

    Ok(())
}
