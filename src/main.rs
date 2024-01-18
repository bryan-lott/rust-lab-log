use chrono::{DateTime, Local};
use clap::{arg, command, value_parser, ArgAction};
use dirs::{config_dir, home_dir};
use rev_lines::RevLines;
use serde::Deserialize;
use std::fs::{read_to_string, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use toml;

#[derive(Deserialize, Debug)]
#[serde(default)]
struct Config {
    default_log_file: PathBuf,
    style: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_log_file: default_log_file_path().into(),
            style: "markdown".to_string(),
        }
    }
}

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

fn determine_headers(path: &File, datetime: DateTime<Local>) -> String {
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
            // We don't have a valid log line at this point???
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                return headers;
            }
        }
    }
    if headers.is_empty() {
        headers.push_str("# Lab Log\n");
        headers.push_str(&year_header(datetime));
        headers.push_str(&day_header(datetime));
    }
    return headers;
}

fn print_last_n_lines(path: File, log_file_path_arg: &str, num_lines: usize) {
    println!(
        "====================| Last {} lines of {} |====================",
        num_lines, log_file_path_arg
    );
    let lines = RevLines::new(path).take(num_lines).collect::<Vec<_>>();
    for line in lines.into_iter().rev() {
        match line {
            Ok(line) => println!("{}", line),
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }
}

fn default_log_file_path() -> std::ffi::OsString {
    home_dir()
        .expect("Couldn't get home directory")
        .join("rlg.md")
        .into_os_string()
}

fn canonicalize_log_file_path(mut config: Config) -> Config {
    let first = config
        .default_log_file
        .components()
        .collect::<Vec<_>>()
        .first()
        .unwrap()
        .as_os_str();
    if first == "~" {
        config.default_log_file = home_dir()
            .expect("Couldn't get home directory")
            .join(config.default_log_file.strip_prefix("~").unwrap())
            .canonicalize()
            .unwrap();
    } else if first == "$HOME" {
        config.default_log_file = home_dir()
            .expect("Couldn't get home directory")
            .join(config.default_log_file.strip_prefix("$HOME").unwrap())
            .canonicalize()
            .unwrap();
    }
    config
}

fn get_config() -> Config {
    match read_to_string(
        config_dir()
            .expect("Couldn't get config directory")
            .join("rlg.toml")
            .as_os_str()
            .to_str()
            .unwrap(),
    ) {
        Ok(config_file_path) => match toml::from_str::<Config>(&config_file_path.to_string()) {
            Ok(config) => canonicalize_log_file_path(config),
            Err(e) => {
                eprintln!("Unable to parse config: {}", e);
                Config::default()
            }
        },
        Err(_) => {
            eprintln!("No config file found, using defaults");
            Config::default()
        }
    }
}

fn main() {
    let config = get_config();
    // Parse the command line args
    let args = command!()
        .arg(
            arg!([file] "Optional file to save the new log entry to")
                .short('f')
                .long("file")
                .value_name("file")
                .required(false)
                .default_value(config.default_log_file.into_os_string())
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!([text] "Text to save to the log" )
                .value_name("text")
                .required(false)
                .default_value("testing!")
                .trailing_var_arg(true)
                .action(ArgAction::Append)
                .value_parser(value_parser!(String)),
        )
        .get_matches();
    let text_arg: String = args
        .get_many::<String>("text")
        .expect("No text provided")
        .cloned()
        .collect::<Vec<String>>()
        .join(" ");
    let log_file_path_arg = args.get_one::<PathBuf>("file").expect("Invalid file");

    // Set our datetime
    let datetime_now = Local::now();
    let datetime_str = datetime_now.format("%Y-%m-%d %H:%M:%S").to_string();
    let mut log_entry = String::new().to_owned();

    // Open and write the log entry to the file
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(log_file_path_arg);
    match file {
        Ok(f) => {
            log_entry.push_str(determine_headers(&f, datetime_now).as_str());
            log_entry.push_str(&format!("- {}: {}", datetime_str, text_arg));
            append_to_file(&f, log_entry);
        }
        Err(e) => {
            println!("Unable to open file: {}", e);
        }
    }

    // Finally, provide feedback on what was just added to the log with context
    print_last_n_lines(
        OpenOptions::new()
            .read(true)
            .open(log_file_path_arg)
            .expect("Unable to read log file"),
        &log_file_path_arg.display().to_string(),
        6,
    );
}

/*
* TODO:
* - [x] Add a ~/.config/rlg.toml config file allowing a custom default log file location
* - [x] Create a new file with appropriate headers if one doesn't exist
* - [x] Only open the file once and pass it around
* - [x] Get default log file working
* - [x] Reverse the printout of last lines
* - [x] Append text to file
* - [x] Add timestamp to file
* - [x] Add day header whenever the day changes
* - [x] Add year header whenever the year changes
*/
