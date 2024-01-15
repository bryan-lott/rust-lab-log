use dirs::home_dir;
use rev_lines::RevLines;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use clap::{arg, command, value_parser, ArgAction};

fn append_to_file(path: &PathBuf, text: String) {
    let file = OpenOptions::new().append(true).create(true).open(path);

    match file {
        Ok(mut f) => {
            if let Err(e) = writeln!(f, "{}", text) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Couldn't open file: {}, creating...", e);
            let mut f = File::create(path).unwrap();
            if let Err(e) = writeln!(f, "{}", text) {
                eprintln!("Couldn't write to new file: {}", e);
            }
        }
    }
}

fn append_headers(path: &PathBuf, datetime: DateTime<Local>) -> String {
    let file = File::open(path);
    let mut headers = String::new();
    match file {
        Ok(f) => {
            let year = datetime.format("%Y").to_string();
            let day = datetime.format("%Y-%m-%d").to_string();

            for line in RevLines::new(f) {
                println!("{}", line.as_ref().unwrap());
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
        Err(e) => {
            eprintln!("Unable to open file, creating with default headers: {} ", e);
            headers.push_str(&format!(
                "# Lab Log\n\n## {}\n\n### {}\n\n",
                datetime.format("%Y"),
                datetime.format("%Y-%m-%d"),
            ));
        }
    }
    return headers;
}

fn main() {
    let default_path: &str = home_dir()
        .and_then(|d| Some(d.join("rlg.md")))
        .unwrap_or_default()
        .as_os_str()
        .to_str()
        .expect("Couldn't convert path to string");

    let args = command!()
        .arg(
            arg!([file] "Optional filename to save the log to")
                .required(false)
                .value_parser(value_parser!(PathBuf))
                .default_value(default_path),
        )
        .arg(
            arg!([text] "Text to save to the log")
                .value_parser(value_parser!(String))
                .trailing_var_arg(true)
                .action(ArgAction::Append)
                .required(true),
        )
        .get_matches();

    let text: String = args
        .get_many::<String>("text")
        .expect("No text provided")
        .cloned()
        .collect::<Vec<String>>()
        .join(" ");
    let datetime = Local::now();
    let file: PathBuf = args.get_one::<PathBuf>("file").unwrap().to_path_buf();

    let mut log_entry = String::new().to_owned();
    log_entry.push_str(append_headers(&file, datetime).as_str());

    let datetime_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    log_entry.push_str(&format!("- {}: {}", datetime_str, text));

    append_to_file(&file, log_entry);
}

/*
* TODO:
* - [ ] Handle the user's home directory correctly
* - [X] Append text to file
* - [X] Add timestamp to file
* - [X] Add day header whenever the day changes
* - [X] Add year header whenever the year changes
*/
