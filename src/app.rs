use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufWriter, Write};
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
            .open(filepath)?;

        for line in existing_lines.iter() {
            writeln!(f, "{}", line)?;
        }

        drop(f);
    }

    // Open file once before processing lines (if not in dry-run mode)
    let mut file_writer = if !args.dry_run {
        Some(BufWriter::new(
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(filepath)?,
        ))
    } else {
        None
    };

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
            writeln!(output_writer, "{}", line)?;
        }

        if let Some(ref mut f) = file_writer {
            writeln!(f, "{}", line)?;
        }
    }

    // Flush and close the file writer
    if let Some(mut f) = file_writer {
        f.flush()?;
    }

    if args.sort && !args.dry_run {
        let mut f = OpenOptions::new()
            .append(false)
            .write(true)
            .truncate(true)
            .create(false)
            .open(&args.filepath)?;

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
            eprintln!("failed to open file for reading: {}", err);

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

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

    #[test]
    fn test_multiple_unique_lines() {
        let input = b"line1\nline2\nline3\n";
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
        app(args, &mut output, &input[..]).unwrap();

        let result_output = String::from_utf8(output).expect("Not UTF-8");
        assert_eq!("line1\nline2\nline3\n", result_output);

        // Verify file contents
        let file_contents = fs::read_to_string(testfile_path).unwrap();
        assert_eq!("line1\nline2\nline3\n", file_contents);
    }

    #[test]
    fn test_quiet_mode() {
        let input = b"line1\nline2\n";
        let testfile = testfile::empty();
        let testfile_path: &std::path::Path = testfile.as_ref();
        let args = Cli {
            quiet_mode: true,
            sort: false,
            trim: false,
            rewrite: false,
            dry_run: false,
            filepath: testfile_path.to_string_lossy().into(),
        };

        let mut output = Vec::new();
        app(args, &mut output, &input[..]).unwrap();

        // Output should be empty in quiet mode
        let result_output = String::from_utf8(output).expect("Not UTF-8");
        assert_eq!("", result_output);

        // But file should still be written
        let file_contents = fs::read_to_string(testfile_path).unwrap();
        assert_eq!("line1\nline2\n", file_contents);
    }

    #[test]
    fn test_trim_whitespace() {
        let input = b"  line1  \n  line2\nline3  \n";
        let testfile = testfile::empty();
        let testfile_path: &std::path::Path = testfile.as_ref();
        let args = Cli {
            quiet_mode: false,
            sort: false,
            trim: true,
            rewrite: false,
            dry_run: false,
            filepath: testfile_path.to_string_lossy().into(),
        };

        let mut output = Vec::new();
        app(args, &mut output, &input[..]).unwrap();

        let result_output = String::from_utf8(output).expect("Not UTF-8");
        assert_eq!("line1\nline2\nline3\n", result_output);

        // Verify file contents are trimmed
        let file_contents = fs::read_to_string(testfile_path).unwrap();
        assert_eq!("line1\nline2\nline3\n", file_contents);
    }

    #[test]
    fn test_sort_lines() {
        let input = b"zebra\napple\n10\n2\n20\n";
        let testfile = testfile::empty();
        let testfile_path: &std::path::Path = testfile.as_ref();
        let args = Cli {
            quiet_mode: false,
            sort: true,
            trim: false,
            rewrite: false,
            dry_run: false,
            filepath: testfile_path.to_string_lossy().into(),
        };

        let mut output = Vec::new();
        app(args, &mut output, &input[..]).unwrap();

        // File should be sorted with natural order
        let file_contents = fs::read_to_string(testfile_path).unwrap();
        assert_eq!("2\n10\n20\napple\nzebra\n", file_contents);
    }

    #[test]
    fn test_dry_run_mode() {
        let input = b"line1\nline2\n";
        let testfile = testfile::empty();
        let testfile_path: &std::path::Path = testfile.as_ref();
        let args = Cli {
            quiet_mode: false,
            sort: false,
            trim: false,
            rewrite: false,
            dry_run: true,
            filepath: testfile_path.to_string_lossy().into(),
        };

        let mut output = Vec::new();
        app(args, &mut output, &input[..]).unwrap();

        // Should output to stdout
        let result_output = String::from_utf8(output).expect("Not UTF-8");
        assert_eq!("line1\nline2\n", result_output);

        // But file should NOT be written
        let file_contents = fs::read_to_string(testfile_path).unwrap_or_default();
        assert_eq!("", file_contents);
    }

    #[test]
    fn test_rewrite_removes_duplicates() {
        let testfile = testfile::empty();
        let testfile_path: &std::path::Path = testfile.as_ref();

        // First, write some lines with duplicates
        fs::write(testfile_path, "line1\nline2\nline1\nline3\nline2\n").unwrap();

        let input = b"";
        let args = Cli {
            quiet_mode: false,
            sort: false,
            trim: false,
            rewrite: true,
            dry_run: false,
            filepath: testfile_path.to_string_lossy().into(),
        };

        let mut output = Vec::new();
        app(args, &mut output, &input[..]).unwrap();

        // File should have duplicates removed
        let file_contents = fs::read_to_string(testfile_path).unwrap();
        assert_eq!("line1\nline2\nline3\n", file_contents);
    }

    #[test]
    fn test_empty_lines_ignored() {
        let input = b"line1\n\nline2\n  \nline3\n";
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
        app(args, &mut output, &input[..]).unwrap();

        let result_output = String::from_utf8(output).expect("Not UTF-8");
        assert_eq!("line1\nline2\nline3\n", result_output);
    }

    #[test]
    fn test_existing_file_with_content() {
        let testfile = testfile::empty();
        let testfile_path: &std::path::Path = testfile.as_ref();

        // Pre-populate file
        fs::write(testfile_path, "existing1\nexisting2\n").unwrap();

        let input = b"existing1\nnew1\nexisting2\nnew2\n";
        let args = Cli {
            quiet_mode: false,
            sort: false,
            trim: false,
            rewrite: false,
            dry_run: false,
            filepath: testfile_path.to_string_lossy().into(),
        };

        let mut output = Vec::new();
        app(args, &mut output, &input[..]).unwrap();

        // Only new lines should be output
        let result_output = String::from_utf8(output).expect("Not UTF-8");
        assert_eq!("new1\nnew2\n", result_output);

        // File should have all unique lines
        let file_contents = fs::read_to_string(testfile_path).unwrap();
        assert_eq!("existing1\nexisting2\nnew1\nnew2\n", file_contents);
    }

    #[test]
    fn test_trim_with_existing_file() {
        let testfile = testfile::empty();
        let testfile_path: &std::path::Path = testfile.as_ref();

        // Pre-populate file with trimmed content
        fs::write(testfile_path, "line1\nline2\n").unwrap();

        let input = b"  line1  \n  line3  \n";
        let args = Cli {
            quiet_mode: false,
            sort: false,
            trim: true,
            rewrite: false,
            dry_run: false,
            filepath: testfile_path.to_string_lossy().into(),
        };

        let mut output = Vec::new();
        app(args, &mut output, &input[..]).unwrap();

        // line1 should be recognized as duplicate, only line3 should be new
        let result_output = String::from_utf8(output).expect("Not UTF-8");
        assert_eq!("line3\n", result_output);
    }

    #[test]
    fn test_sort_with_existing_content() {
        let testfile = testfile::empty();
        let testfile_path: &std::path::Path = testfile.as_ref();

        // Pre-populate file
        fs::write(testfile_path, "zebra\napple\n").unwrap();

        let input = b"banana\n";
        let args = Cli {
            quiet_mode: false,
            sort: true,
            trim: false,
            rewrite: false,
            dry_run: false,
            filepath: testfile_path.to_string_lossy().into(),
        };

        let mut output = Vec::new();
        app(args, &mut output, &input[..]).unwrap();

        // File should be sorted with all content
        let file_contents = fs::read_to_string(testfile_path).unwrap();
        assert_eq!("apple\nbanana\nzebra\n", file_contents);
    }

    #[test]
    fn test_combined_trim_and_sort() {
        let input = b"  zebra  \n  apple\nbanana  \n";
        let testfile = testfile::empty();
        let testfile_path: &std::path::Path = testfile.as_ref();
        let args = Cli {
            quiet_mode: false,
            sort: true,
            trim: true,
            rewrite: false,
            dry_run: false,
            filepath: testfile_path.to_string_lossy().into(),
        };

        let mut output = Vec::new();
        app(args, &mut output, &input[..]).unwrap();

        // File should be trimmed and sorted
        let file_contents = fs::read_to_string(testfile_path).unwrap();
        assert_eq!("apple\nbanana\nzebra\n", file_contents);
    }
}

