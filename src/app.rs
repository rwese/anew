use std::fs::{File, OpenOptions};
use std::io::{BufRead, Write};
use std::{fs, io};

use clap::Parser;
use indexmap::IndexSet;

#[derive(Parser, Debug, Clone)]
#[command(author = "wese", version, about = "simple tool to write non-duplicate lines to a file", long_about = None)]
pub struct Cli {
    #[arg(short, long, help = "Do not output new lines to stdout")]
    pub quiet_mode: bool,

    #[arg(short, long, help = "Sort lines (natsort)")]
    pub sort: bool,

    #[arg(short, long, help = "Trim whitespaces")]
    pub trim: bool,

    #[arg(
        short,
        long,
        help = "Rewrite existing destination file to remove duplicates"
    )]
    pub rewrite: bool,

    #[arg(long, help = "Do not write to file, only output what would be written")]
    pub dry_run: bool,

    #[arg(help = "Destination file")]
    pub filepath: String,
}

pub fn args() -> Cli {
    Cli::parse()
}

pub fn app<R, W>(
    args: Cli,
    mut output_writer: W,
    input_reader: R,
) -> Result<(), Box<dyn std::error::Error>>
where
    R: BufRead,
    W: Write,
{
    let filepath = &args.filepath;

    let mut existing_lines = match load_file(&args) {
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

        for line in existing_lines.iter() {
            writeln!(f, "{}", line).expect("failed to write to file");
        }

        drop(f);
    }

    let input_lines = input_reader.lines();
    for line in input_lines {
        let line = line?;
        let line = if args.trim {
            line.trim()
        } else {
            line.as_str()
        };
        if !is_newline(&existing_lines, line) {
            continue;
        }

        existing_lines.insert(line.to_string());

        if !args.quiet_mode {
            writeln!(output_writer, "{}", line).expect("failed to write to output");
        }

        if !args.dry_run {
            let mut f: Option<File> = None;

            if f.is_none() {
                f = Some(
                    OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(filepath)
                        .expect("failed to open file"),
                );
            }

            let mut f = f.unwrap();
            writeln!(f, "{}", line).expect("failed to write to file");

            drop(f);
        }
    }

    if args.sort && !args.dry_run {
        let mut f = OpenOptions::new()
            .append(false)
            .write(true)
            .create(false)
            .open(&args.filepath)
            .expect("failed to open file");

        existing_lines.sort_by(|a, b| natord::compare_ignore_case(a.as_str(), b.as_str()));

        for line in existing_lines.iter() {
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
                let line = if args.trim { line.trim() } else { line };
                if !is_newline(&lines, line) {
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

fn is_newline(lines: &IndexSet<String>, line: &str) -> bool {
    if line.trim() == "" {
        return false;
    }

    if lines.contains(line) {
        return false;
    }

    true
}

#[test]
fn test_basic_write_twice_the_same() {
    let input = b"I'm George1\n";
    let testfile = testfile::empty();
    let testfile_path: &std::path::Path = testfile.as_ref();
    let args = Cli {
        quiet_mode: false,
        sort: false,
        trim: false,
        rewrite: false,
        dry_run: false,
        filepath: testfile_path.to_string_lossy().into(),
    };

    let mut output = Vec::new();
    let result = app(args.clone(), &mut output, &input[..]);
    match result {
        Ok(_) => {}
        Err(err) => panic!("{}", err),
    }

    let result_output = String::from_utf8(output).expect("Not UTF-8");
    assert_eq!(input, result_output.as_bytes());

    let mut output = Vec::new();
    let result = app(args.clone(), &mut output, &input[..]);
    match result {
        Ok(_) => {}
        Err(err) => panic!("{}", err),
    }

    let result_output = String::from_utf8(output).expect("Not UTF-8");
    assert_eq!("".as_bytes(), result_output.as_bytes());
}

