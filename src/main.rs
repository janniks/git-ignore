use std::collections::HashSet;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{stdout, Write};

use regex::Regex;

use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode as Key, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{self, ClearType::CurrentLine},
};
use itertools::Itertools;
use serde::Deserialize;
use sublime_fuzzy::best_match;

const SHOW_LINES: usize = 4;
const TEMPLATES_URL: &str = "https://api.github.com/repos/toptal/gitignore/contents/templates";
const IGNORE_URL: &str = "https://www.toptal.com/developers/gitignore/api/";

#[derive(Deserialize, Debug)]
struct File {
    name: String,
}

#[derive(Debug)]
struct Item<'a> {
    name: &'a String,
    score: isize,
}

#[derive(PartialEq)]
enum Action {
    Accept,
    Cancel,
    Continue,
}

fn get_ignores() -> Result<Vec<String>, Box<dyn Error>> {
    let files = minreq::get(TEMPLATES_URL)
        .with_header("User-Agent", "git-ignore")
        .send()?
        .json::<Vec<File>>()?;

    let re = Regex::new(r"\.(patch|gitignore)")?;
    let mut files: Vec<String> = files
        .iter()
        .map(|f| re.replace_all(&f.name, "").to_string())
        .collect();
    files.dedup();
    Ok(files)
}

fn filter_fuzzy<'a>(
    source: &'a Vec<String>,
    word: &String,
    blocklist: &HashSet<&String>,
) -> Vec<&'a String> {
    if word.is_empty() {
        return Vec::new();
    }

    let mut items = source
        .iter()
        .filter(|i| !blocklist.contains(i))
        .map(|s| Item {
            name: s,
            score: match best_match(&word, &s) {
                Some(r) => r.score(),
                None => 0,
            },
        })
        .collect::<Vec<Item>>();

    items.sort_unstable_by(|b, a| a.score.partial_cmp(&b.score).unwrap());
    items
        .iter()
        .filter(|i| i.score > 0) // todo: adjust for more/less matches
        .map(|i| i.name)
        .take(SHOW_LINES as usize)
        .collect::<Vec<&String>>()
}

fn render(
    arrow: usize,
    filtered_items: &Vec<&String>,
    chosen_items: &HashSet<&String>,
    typed: &String,
) {
    let mut stdout = stdout();

    if !chosen_items.is_empty() {
        execute!(stdout, cursor::MoveUp(1), terminal::Clear(CurrentLine)).unwrap();
        write!(
            stdout,
            "\rSelected templates: {}\n",
            chosen_items.iter().join(", ")
        )
        .unwrap();
    } else {
        execute!(stdout, terminal::Clear(CurrentLine)).unwrap();
    }
    execute!(stdout, terminal::Clear(CurrentLine)).unwrap();
    write!(stdout, "\rSearch ignore templates: {}", typed).unwrap();
    execute!(stdout, cursor::SavePosition).unwrap();
    write!(stdout, "\n\r").unwrap();

    // render visible items
    let mut printed_lines = 0;
    for item in filtered_items {
        write!(
            stdout,
            "\r {} ",
            if printed_lines == arrow { '>' } else { ' ' }
        )
        .unwrap();

        write!(stdout, "{}\n", item).unwrap();
        printed_lines += 1;
    }

    // print empty lines
    for _ in 0..(SHOW_LINES - filtered_items.len()) {
        execute!(stdout, terminal::Clear(CurrentLine)).unwrap();
        write!(stdout, "\n").unwrap();
    }

    // restore
    execute!(stdout, cursor::RestorePosition).unwrap();
    stdout.lock().flush().unwrap();
}

fn write_to_file(chosen_items: HashSet<&String>) {
    let ignore_url = format!("{}/{}", IGNORE_URL, chosen_items.iter().join(","));
    let response = minreq::get(ignore_url)
        .send()
        .expect("\n\r! Unable to get ignore file");
    let body = response.as_str().expect("\n\r! Unable read body");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(".gitignore")
        .expect("\n\r! Unable to open file options");
    write!(file, "{}", body).expect("\n\r! Unable to write to file");
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Loading ignore templates from GitHub...");
    let ignores = get_ignores().expect("\n\r! Unable to load templates from GitHub");

    let mut stdout = std::io::stdout();

    for _ in 0..10 {
        print!("\n");
    }
    stdout.flush().unwrap();

    let (x, y) = cursor::position().unwrap();
    execute!(stdout, cursor::MoveTo(x, y - 10));

    let mut stdout = std::io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, event::DisableMouseCapture);

    // loop variables
    // todo: extract loop to separate method
    let mut state = Action::Continue;

    let mut arrow: usize = 0;
    let mut typed = String::new();

    let mut chosen_items: HashSet<&String> = HashSet::new();
    let mut filtered_items: Vec<&String> = Vec::new();

    loop {
        render(arrow, &filtered_items, &chosen_items, &typed);

        if state != Action::Continue {
            break;
        }

        let event = read()?;
        if let Event::Key(KeyEvent {
            code: key,
            modifiers: KeyModifiers::NONE,
        }) = event
        {
            match key {
                Key::Up => {
                    if 0 < arrow {
                        arrow -= 1;
                    }
                }
                Key::Down => {
                    if arrow < SHOW_LINES - 1 {
                        arrow += 1;
                    }
                }
                Key::Tab => {
                    if typed.is_empty() {
                        state = Action::Accept;
                    }
                }
                Key::Enter => {
                    if typed.is_empty() {
                        if filtered_items.is_empty() {
                            state = Action::Cancel;
                        } else {
                            state = Action::Accept;
                        }
                    } else if let Some(selected) = filtered_items.get(arrow as usize) {
                        chosen_items.insert(selected);
                        typed = String::new();
                    }

                    filtered_items = Vec::new();
                    arrow = 0;
                }
                Key::Backspace => {
                    if !typed.is_empty() {
                        // todo: maybe cancel if empty
                        typed.pop();
                        filtered_items = filter_fuzzy(&ignores, &typed, &chosen_items);
                    }
                }
                Key::Char(character) if character.is_alphanumeric() => {
                    typed.push(character);
                    arrow = 0;
                    filtered_items = filter_fuzzy(&ignores, &typed, &chosen_items);
                }
                _ => {
                    // cancel
                    typed = String::new();
                    filtered_items = Vec::new();
                    arrow = 0;

                    state = Action::Cancel
                }
            }
        } else if let Event::Key(KeyEvent {
            code: _,
            modifiers: _,
        }) = event
        {
            // cancel
            typed = String::new();
            filtered_items = Vec::new();
            arrow = 0;

            state = Action::Cancel
        }
    }

    execute!(stdout, terminal::Clear(CurrentLine)).unwrap();
    write!(stdout, "\r").unwrap();

    match state {
        Action::Cancel => {
            write!(stdout, "Canceled\n\r").unwrap();
        }
        Action::Accept => {
            write!(stdout, "Writing to .gitignore file\n\r").unwrap();
            write_to_file(chosen_items);
            write!(stdout, "Done\n\r").unwrap();
        }
        _ => (),
    }

    terminal::disable_raw_mode()?;

    Ok(())
}
