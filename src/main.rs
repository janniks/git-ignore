#![allow(non_upper_case_globals)]

use std::collections::HashSet;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;

use regex::Regex;

use crossterm::{
    cursor::{self, RestorePosition, SavePosition},
    event::{self, read, Event, KeyCode as Key, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, disable_raw_mode, enable_raw_mode, ClearType::CurrentLine},
};
use itertools::Itertools;
use serde::Deserialize;
use sublime_fuzzy::best_match;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

// config
const SHOW_LINES: usize = 4;
const TEMPLATES_URL: &str = "https://api.github.com/repos/toptal/gitignore/contents/templates";
const IGNORE_URL: &str = "https://www.toptal.com/developers/gitignore/api/";

// crossterm shortconsts
const ClearLine: terminal::Clear = terminal::Clear(CurrentLine);
const MoveToPreviousLine: cursor::MoveToPreviousLine = cursor::MoveToPreviousLine(1);
const MoveToNextLine: cursor::MoveToNextLine = cursor::MoveToNextLine(1);
const BlueColor: crossterm::style::SetForegroundColor = SetForegroundColor(Color::Blue);
const GreenColor: crossterm::style::SetForegroundColor = SetForegroundColor(Color::Green);

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

fn get_ignores() -> Result<Vec<String>> {
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
    let mut stdout = std::io::stdout();

    queue!(stdout, ClearLine, ResetColor).unwrap();

    if !chosen_items.is_empty() {
        queue!(
            stdout,
            MoveToPreviousLine,
            ClearLine,
            Print(format!(
                "Selected templates: {}{}{}",
                BlueColor,
                chosen_items.iter().join(", "),
                ResetColor
            )),
            MoveToNextLine,
            ClearLine
        )
        .unwrap();
    }

    let search_text = if chosen_items.is_empty() {
        format!("\rSearch templates: {}{}{}", GreenColor, typed, ResetColor)
    } else {
        format!(
            "\rSearch additional templates: {}{}{}",
            GreenColor, typed, ResetColor
        )
    };

    queue!(
        stdout,
        Print(search_text),
        SavePosition,
        MoveToNextLine,
        ClearLine
    )
    .unwrap();

    // render visible items
    for (i, item) in filtered_items.iter().enumerate() {
        write!(
            stdout,
            "\r {} {}\n",
            if i == arrow {
                format!("{}>{}", BlueColor, ResetColor)
            } else {
                " ".to_string()
            },
            item
        )
        .unwrap();
    }

    // print empty lines
    for _ in 0..(SHOW_LINES - filtered_items.len()) {
        queue!(stdout, ClearLine, Print("\n")).unwrap();
    }

    // restore
    queue!(stdout, RestorePosition, GreenColor).unwrap();
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

fn create_screen_estate() -> Result<()> {
    let mut stdout = std::io::stdout();
    for _ in 0..SHOW_LINES + 1 {
        write!(stdout, "\n")?;
    }
    execute!(stdout, cursor::MoveUp(SHOW_LINES as u16 + 1))?;
    Ok(())
}

fn main() -> Result<()> {
    println!("Loading ignore templates from GitHub...",);
    let ignores = get_ignores().expect("\n\r! Unable to load templates from GitHub");

    println!(
        "{}Type to search for ignore templates{}",
        BlueColor, ResetColor
    );
    println!(" - ENTER to select a template or when you're done choosing");
    println!(" - ESC to cancel at any time\n");

    create_screen_estate()?;
    enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    execute!(stdout, event::DisableMouseCapture).unwrap();

    // loop variables
    // todo: extract loop to separate method
    let mut state = Action::Continue;

    let mut arrow: usize = 0;
    let mut typed = String::new();

    let mut chosen_items: HashSet<&String> = HashSet::new();
    let mut filtered_items: Vec<&String> = Vec::new();

    while state == Action::Continue {
        render(arrow, &filtered_items, &chosen_items, &typed);

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
                        if chosen_items.is_empty() {
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

    execute!(stdout, ClearLine, ResetColor, Print("\r")).unwrap();

    match state {
        Action::Cancel => {
            execute!(stdout, Print("Canceled\n\r")).unwrap();
        }
        Action::Accept => {
            execute!(stdout, Print("Writing to .gitignore file\n\r")).unwrap();
            write_to_file(chosen_items);
            execute!(stdout, Print("Done\n\r")).unwrap();
        }
        _ => (),
    }

    disable_raw_mode()?;

    Ok(())
}
