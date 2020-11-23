use std::collections::HashSet;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{stdout, Write};

use regex::Regex;

use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode as Key, KeyEvent, KeyModifiers},
    execute, queue,
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
        queue!(
            stdout,
            cursor::MoveToPreviousLine(1),
            terminal::Clear(CurrentLine),
            Print(format!(
                "Selected templates: {}",
                chosen_items.iter().join(", ")
            ))
        );
    } else {
        // queue!(stdout, terminal::Clear(CurrentLine));
    }

    queue!(
        stdout,
        cursor::MoveToNextLine(1),
        terminal::Clear(CurrentLine),
        Print(format!("Search ignore templates: {}", typed)),
        cursor::SavePosition,
        cursor::MoveToNextLine(1)
    );

    // render visible items
    for (i, item) in filtered_items.iter().enumerate() {
        queue!(
            stdout,
            Print(format!(
                " {} {}",
                if i == arrow as usize { '>' } else { ' ' },
                item
            )),
            cursor::MoveToNextLine(1)
        );
    }

    // print empty lines
    for _ in 0..(SHOW_LINES - filtered_items.len()) {
        queue!(stdout, cursor::MoveToNextLine(1));
    }

    // restore
    queue!(stdout, cursor::RestorePosition);
    stdout.flush().unwrap();
}

fn write_to_file(chosen_items: HashSet<&String>) {
    let ignore_url = format!("{}/{}", IGNORE_URL, chosen_items.iter().join(","));
    let response = minreq::get(ignore_url)
        .send()
        .expect("Unable to get ignore file");
    let body = response.as_str().expect("Unable read body");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(".gitignore")
        .expect("Unable to open file options");
    write!(file, "{}", body).expect("Unable to write to file");
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Loading ignore templates from GitHub...");
    let ignores = get_ignores().expect("Unable to load templates from GitHub");

    let mut stdout = stdout();

    let (x, y) = cursor::position().unwrap();
    println!("pos {} {}", x, y);

    terminal::enable_raw_mode().expect("Unable to enter raw mode");
    execute!(stdout, event::DisableMouseCapture);

    execute!(
        stdout,
        cursor::SavePosition,
        terminal::ScrollUp(10),
        cursor::RestorePosition
    );

    let (x, y) = terminal::size().unwrap();
    println!("siz {} {}", x, y);
    execute!(stdout, terminal::SetSize(x, y + SHOW_LINES as u16));
    let (x, y) = terminal::size().unwrap();
    println!("siz {} {}", x, y);

    // loop variable
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
                        state = Action::Accept;
                    // todo: write \r or restore for better cursor UX after ENTER
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
                Key::Char(character) => {
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

    queue!(stdout, cursor::MoveToNextLine(1));

    match state {
        Action::Cancel => {
            queue!(stdout, Print("Canceled"), cursor::MoveToNextLine(1));
        }
        Action::Accept => {
            execute!(
                stdout,
                Print("Writing to .gitignore file"),
                cursor::MoveToNextLine(1)
            );
            write_to_file(chosen_items);
            queue!(stdout, Print("Done"), cursor::MoveToNextLine(1)).unwrap();
        }
        _ => (),
    }

    stdout.flush().unwrap();

    terminal::disable_raw_mode()?;

    Ok(())
}
