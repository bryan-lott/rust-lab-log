use dirs::home_dir;
use rev_lines::RevLines;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use clap::{arg, command, value_parser, ArgAction};

/*
#[derive(Parser)]
struct Cli {
    /// Optional filename to save the log to
    #[arg(short, long, default_value = "/Users/blott/Dropbox/notes/rlg.md")]
    file: Option<std::path::PathBuf>,

    /// Text to save to the log
    #[arg(trailing_var_arg = true)]
    text: Vec<String>,
}
*/

fn append_to_file(mut path: &File, text: String) {
    if let Err(e) = writeln!(path, "{}", text) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

fn year_header(datetime: DateTime<Local>) -> String {
    format!("\n## {}", datetime.format("%Y"))
}

fn day_header(datetime: DateTime<Local>) -> String {
    format!("\n\n### {}\n", datetime.format("%Y-%m-%d"))
}

fn append_headers(path: &File, datetime: DateTime<Local>) -> String {
    let mut headers = String::new();
    let year = datetime.format("%Y").to_string();
    let day = datetime.format("%Y-%m-%d").to_string();

    for line in RevLines::new(path) {
        match line {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                if line.starts_with(format!("- {}", day).as_str()) {
                    return headers;
                }
                if !line.starts_with(format!("- {}", year).as_str()) {
                    headers.push_str(&year_header(datetime));
                }
                headers.push_str(&day_header(datetime));
                return headers;
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }
    if headers.is_empty() {
        headers.push_str("# Lab Log\n");
        headers.push_str(&year_header(datetime));
        headers.push_str(&day_header(datetime));
    }
    return headers;
}

fn print_last_n_lines(path: File, num_lines: usize) {
    println!(
        "====================| Last {} lines |====================",
        num_lines
    );
    let lines = RevLines::new(path).take(num_lines).collect::<Vec<_>>();
    for line in lines.into_iter().rev() {
        match line {
            Ok(line) => println!("{}", line),
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }
}

fn main() {
    // let args = Cli::parse();

    let mut default_log_file = home_dir().expect("Couldn't get home directory");
    default_log_file.push("rlg.md");
    let file = default_log_file.as_mut_os_string();

    // let default_log_file = "/Users/blott/Dropbox/notes/rlg.md";
    let args = command!()
        .arg(
            arg!([file] "Optional file to save the new log entry to")
                .short('f')
                .long("file")
                .value_name("file")
                .required(false)
                .default_value(file.clone())
                // .default_value("/Users/blott/Dropbox/notes/rlg.md")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!([text] "Text to save to the log" )
                .value_name("text")
                .required(true)
                .trailing_var_arg(true)
                .action(ArgAction::Append)
                .value_parser(value_parser!(String)),
        )
        .get_matches();
    let text: String = args
        .get_many::<String>("text")
        .expect("No text provided")
        .cloned()
        .collect::<Vec<String>>()
        .join(" ");
    // let text = args.text.join(" ");
    let datetime = Local::now();
    let datetime_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    let mut log_entry = String::new().to_owned();

    let file_path = args.get_one::<PathBuf>("file").expect("Invalid file");
    let file = OpenOptions::new().append(true).create(true).open(file_path);
    match file {
        Ok(f) => {
            log_entry.push_str(append_headers(&f, datetime).as_str());
            log_entry.push_str(&format!("- {}: {}", datetime_str, text));
            append_to_file(&f, log_entry);
        }
        Err(e) => {
            println!("Unable to open file: {}", e);
        }
    }
    print_last_n_lines(
        OpenOptions::new()
            .read(true)
            .open(file_path)
            .expect("Unable to read log file"),
        6,
    );
}

/*
* TODO:
* - [ ] Add a ~/.config/rlg.toml config file allowing a custom default log file location
* - [x] Create a new file with appropriate headers if one doesn't exist
* - [x] Only open the file once and pass it around
* - [x] Get default log file working
* - [x] Reverse the printout of last lines
* - [x] Append text to file
* - [x] Add timestamp to file
* - [x] Add day header whenever the day changes
* - [x] Add year header whenever the year changes
*/
