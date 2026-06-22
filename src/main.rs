use chrono::{DateTime, Local};
use clap::{arg, command, value_parser, ArgAction};
use dirs::{config_dir, home_dir};
use rev_lines::RevLines;
use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, write, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
struct Config {
    default_log_file: PathBuf,
    todo_file: PathBuf,
    style: String,
    last_n_lines: usize,
}

impl Default for Config {
    fn default() -> Self {
        let home = home_dir().expect("Couldn't get home directory");
        Self {
            default_log_file: home.join("rlg.md"),
            todo_file: home.join("rlg-todos.toml"),
            style: "markdown".to_string(),
            last_n_lines: 6,
        }
    }
}

fn canonicalize_path(path: PathBuf) -> PathBuf {
    match path.components().next().and_then(|c| c.as_os_str().to_str()) {
        Some("~") => home_dir()
            .expect("Couldn't get home directory")
            .join(path.strip_prefix("~").unwrap()),
        Some("$HOME") => home_dir()
            .expect("Couldn't get home directory")
            .join(path.strip_prefix("$HOME").unwrap()),
        _ => path,
    }
}

fn get_config() -> Config {
    let config_path = config_dir()
        .expect("Couldn't get config directory")
        .join("rlg.toml");
    match read_to_string(&config_path) {
        Ok(s) => match toml::from_str::<Config>(&s) {
            Ok(mut config) => {
                config.default_log_file = canonicalize_path(config.default_log_file);
                config.todo_file = canonicalize_path(config.todo_file);
                config
            }
            Err(e) => {
                eprintln!("Unable to parse config: {}", e);
                Config::default()
            }
        },
        Err(_) => {
            println!("No config file found, creating default config at {}", config_path.display());
            let config = Config::default();
            if let Err(e) = write(&config_path, toml::to_string_pretty(&config).unwrap()) {
                eprintln!("Unable to write default config: {}", e);
            }
            config
        }
    }
}

// --- Todo ---

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
enum Status {
    Open,
    Active,
    Done,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Todo {
    id: usize,
    text: String,
    status: Status,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Todos {
    #[serde(default)]
    todos: Vec<Todo>,
}

fn load_todos(path: &PathBuf) -> Todos {
    match read_to_string(path) {
        Ok(s) => toml::from_str(&s).unwrap_or_default(),
        Err(_) => Todos::default(),
    }
}

fn save_todos(path: &PathBuf, todos: &mut Todos) {
    // ponytail: stable-sort keeps relative order within each group
    todos.todos.sort_by_key(|t| t.status == Status::Done);
    write(path, toml::to_string_pretty(todos).unwrap()).expect("Couldn't write todos file");
}

fn print_todos(todos: &Todos, show_done: bool) {
    let visible: Vec<_> = todos
        .todos
        .iter()
        .filter(|t| show_done || t.status != Status::Done)
        .collect();
    if visible.is_empty() {
        return;
    }
    println!("Todos:");
    for t in &visible {
        let marker = match t.status {
            Status::Open => "[ ]",
            Status::Active => "[~]",
            Status::Done => "[x]",
        };
        println!("  {} {}  {}", marker, t.id, t.text);
    }
    println!();
}

fn todo_add(todos: &mut Todos, text: &str) -> String {
    let id = todos.todos.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    todos.todos.push(Todo { id, text: text.to_string(), status: Status::Open });
    format!("TODO created: {} [#{}]", text, id)
}

fn todo_transition(todos: &mut Todos, id: usize, status: Status) -> Option<String> {
    let label = match status {
        Status::Active => "in-progress",
        Status::Done => "completed",
        Status::Open => "reopened",
    };
    todos.todos.iter_mut().find(|t| t.id == id).map(|t| {
        let msg = format!("TODO {}: {} [#{}]", label, t.text, id);
        t.status = status;
        msg
    }).or_else(|| { eprintln!("Todo #{} not found", id); None })
}

fn todo_rm(todos: &mut Todos, id: usize) {
    let before = todos.todos.len();
    todos.todos.retain(|t| t.id != id);
    if todos.todos.len() == before {
        eprintln!("Todo #{} not found", id);
    }
}

fn todo_reword(todos: &mut Todos, id: usize, new_text: &str) -> Option<String> {
    todos.todos.iter_mut().find(|t| t.id == id).map(|t| {
        let old = t.text.clone();
        t.text = new_text.to_string();
        format!("TODO reworded: {} → {} [#{}]", old, new_text, id)
    }).or_else(|| { eprintln!("Todo #{} not found", id); None })
}

// --- Log ---

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
    headers
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

fn append_to_file(mut path: &File, text: String) {
    if let Err(e) = writeln!(path, "{}", text) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

fn open_log_file(file_path: &PathBuf) -> Result<File, std::io::Error> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open(file_path)
}

fn write_to_log_file(file_path: &PathBuf, text: Option<&str>) -> Result<(), std::io::Error> {
    let datetime_now = Local::now();
    let file = open_log_file(file_path)?;
    let mut log_entry = String::new();

    log_entry.push_str(determine_headers(&file, datetime_now).as_str());
    if let Some(entry_text) = text {
        let datetime_str = datetime_now.format("%Y-%m-%d %H:%M:%S").to_string();
        log_entry.push_str(&format!("- {}: {}", datetime_str, entry_text));
    }
    append_to_file(&file, log_entry);
    if text.is_none() {
        println!("Created new log file: {}", file_path.display());
    }
    Ok(())
}

fn show_dashboard(log_file_path: &PathBuf, todo_file: &PathBuf, last_n_lines: usize) {
    let todos = load_todos(todo_file);
    print_todos(&todos, false);
    match OpenOptions::new().read(true).open(log_file_path) {
        Ok(file) => print_last_n_lines(file, &log_file_path.display().to_string(), last_n_lines),
        Err(_) => {
            if let Err(e) = write_to_log_file(log_file_path, None) {
                eprintln!("Unable to create log file: {}", e);
            }
        }
    }
}

fn main() {
    let config = get_config();

    let mut argv: Vec<String> = std::env::args().collect();
    if std::path::Path::new(&argv[0])
        .file_stem()
        .and_then(|s| s.to_str())
        == Some("rtd")
    {
        argv.insert(1, "todo".to_string());
    }

    let args = command!()
        .arg(
            arg!([file] "Optional file to save the new log entry to")
                .short('f')
                .long("file")
                .value_name("file")
                .required(false)
                .default_value(config.default_log_file.clone().into_os_string())
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!([text] "Text to save to the log")
                .value_name("text")
                .required(false)
                .trailing_var_arg(true)
                .action(ArgAction::Append)
                .value_parser(value_parser!(String)),
        )
        .subcommand(
            command!("todo")
                .alias("t")
                .about("Manage todos")
                .arg(
                    arg!([text] "Add a new todo (shorthand for 'todo add')")
                        .trailing_var_arg(true)
                        .action(ArgAction::Append)
                        .value_parser(value_parser!(String)),
                )
                .subcommand(
                    command!("ls")
                        .about("List todos")
                        .arg(arg!(--all "Include done items")),
                )
                .subcommand(
                    command!("start")
                        .about("Mark todo in-progress")
                        .arg(arg!(<id> "Todo ID").value_parser(value_parser!(usize))),
                )
                .subcommand(
                    command!("done")
                        .about("Mark todo complete")
                        .arg(arg!(<id> "Todo ID").value_parser(value_parser!(usize))),
                )
                .subcommand(
                    command!("rm")
                        .about("Remove a todo")
                        .arg(arg!(<id> "Todo ID").value_parser(value_parser!(usize))),
                )
                .subcommand(
                    command!("reword")
                        .about("Reword a todo")
                        .arg(arg!(<id> "Todo ID").value_parser(value_parser!(usize)))
                        .arg(
                            arg!([text] "New text for the todo")
                                .trailing_var_arg(true)
                                .action(ArgAction::Append)
                                .value_parser(value_parser!(String)),
                        ),
                ),
        )
        .get_matches_from(&argv);

    let log_file_path = args.get_one::<PathBuf>("file").expect("Invalid file");

    if let Some(todo_matches) = args.subcommand_matches("todo") {
        let mut todos = load_todos(&config.todo_file);

        match todo_matches.subcommand() {
            Some(("ls", sub)) => {
                print_todos(&todos, sub.get_flag("all"));
                return;
            }
            Some(("rm", sub)) => {
                let id = *sub.get_one::<usize>("id").unwrap();
                todo_rm(&mut todos, id);
                save_todos(&config.todo_file, &mut todos);
                print_todos(&todos, false);
                return;
            }
            Some(("start", sub)) => {
                let id = *sub.get_one::<usize>("id").unwrap();
                if let Some(msg) = todo_transition(&mut todos, id, Status::Active) {
                    save_todos(&config.todo_file, &mut todos);
                    let _ = write_to_log_file(log_file_path, Some(&msg));
                }
            }
            Some(("done", sub)) => {
                let id = *sub.get_one::<usize>("id").unwrap();
                if let Some(msg) = todo_transition(&mut todos, id, Status::Done) {
                    save_todos(&config.todo_file, &mut todos);
                    let _ = write_to_log_file(log_file_path, Some(&msg));
                }
            }
            Some(("reword", sub)) => {
                let id = *sub.get_one::<usize>("id").unwrap();
                if let Some(text_vals) = sub.get_many::<String>("text") {
                    let new_text = text_vals.cloned().collect::<Vec<_>>().join(" ");
                    if let Some(msg) = todo_reword(&mut todos, id, &new_text) {
                        save_todos(&config.todo_file, &mut todos);
                        let _ = write_to_log_file(log_file_path, Some(&msg));
                    }
                }
            }
            _ => {
                if let Some(text_vals) = todo_matches.get_many::<String>("text") {
                    let text = text_vals.cloned().collect::<Vec<_>>().join(" ");
                    let msg = todo_add(&mut todos, &text);
                    save_todos(&config.todo_file, &mut todos);
                    let _ = write_to_log_file(log_file_path, Some(&msg));
                }
                // rlg todo with no args → fall through to dashboard
            }
        }

        show_dashboard(log_file_path, &config.todo_file, config.last_n_lines);
        return;
    }

    // Log entry
    if args.contains_id("text") {
        let text_arg: String = args
            .get_many::<String>("text")
            .expect("No text provided")
            .cloned()
            .collect::<Vec<String>>()
            .join(" ");
        if let Err(e) = write_to_log_file(log_file_path, Some(&text_arg)) {
            eprintln!("Unable to add log entry: {}", e);
        }
    }

    show_dashboard(log_file_path, &config.todo_file, config.last_n_lines);
}

/*
* TODO:
* - [x] If no args are provided, spit out the last n-lines of the log
* - [x] Todo management (rlg todo "text" / ls / start / done / rm)
* - [ ] Create a minimal TUI
*   - [ ] Add creation of new log lines
*   - [ ] Add view of existing log lines
*   - [ ] Add deletion of existing log lines
* - [x] Create a default config if one isn't found. Let the user know.
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
