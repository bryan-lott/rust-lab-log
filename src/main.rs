use rev_lines::RevLines;
use std::fs::{File, OpenOptions};
use std::io::Write;

use chrono::{DateTime, Local};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Optional filename to save the log to
    #[arg(short, long, default_value = "/Users/blott/Dropbox/notes/rlg.md")]
    file: Option<std::path::PathBuf>,

    /// Text to save to the log
    #[arg(trailing_var_arg = true)]
    text: Vec<String>,
}

fn append_to_file(path: &std::path::PathBuf, text: String) {
    let file = OpenOptions::new().append(true).create(true).open(path);

    match file {
        Ok(mut f) => {
            if let Err(e) = writeln!(f, "{}", text) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Couldn't open file: {}", e);
        }
    }
}

fn append_headers(path: &std::path::PathBuf, datetime: DateTime<Local>) -> String {
    let file = File::open(path);
    let mut headers = String::new();
    match file {
        Ok(f) => {
            let year = datetime.format("%Y").to_string();
            let day = datetime.format("%Y-%m-%d").to_string();

            for line in RevLines::new(f) {
                match line {
                    Ok(line) => {
                        if line.trim().is_empty() {
                            continue;
                        }
                        if line.starts_with(format!("- {}", day).as_str()) {
                            return headers;
                        }
                        if !line.starts_with(format!("- {}", year).as_str()) {
                            headers.push_str(&format!("\n\n## {}", datetime.format("%Y")));
                        }
                        headers.push_str(&format!("\n\n### {}\n\n", datetime.format("%Y-%m-%d")));
                        return headers;
                    }
                    Err(e) => eprintln!("Error reading line: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Couldn't open file: {}", e),
    }
    return headers;
}

fn print_last_n_lines(path: &std::path::PathBuf, num_lines: usize) {
    let file = File::open(path);
    match file {
        Ok(f) => {
            println!(
                "====================| Last {} lines |====================",
                num_lines
            );
            let lines = RevLines::new(f).take(num_lines).collect::<Vec<_>>();
            for line in lines.into_iter().rev() {
                match line {
                    Ok(line) => println!("{}", line),
                    Err(e) => eprintln!("Error reading line: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Couldn't open file: {}", e),
    }
}

fn main() {
    let args = Cli::parse();
    let text = args.text.join(" ");
    let datetime = Local::now();

    let mut log_entry = String::new().to_owned();
    log_entry.push_str(append_headers(&args.file.as_ref().unwrap(), datetime).as_str());

    let datetime_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    log_entry.push_str(&format!("- {}: {}", datetime_str, text));

    append_to_file(args.file.as_ref().unwrap(), log_entry);
    print_last_n_lines(&args.file.as_ref().unwrap(), 5);
}

/*
* TODO:
* - [x] Reverse the printout of last lines
* - [x] Append text to file
* - [x] Add timestamp to file
* - [x] Add day header whenever the day changes
* - [x] Add year header whenever the year changes
*/
