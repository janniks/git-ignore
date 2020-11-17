use std::collections::HashSet;
use std::io::{self, Write};
use std::thread;
use std::time;

use regex::Regex;

use serde::Deserialize;
use termion;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;

const SHOW_LINES: u8 = 3;

#[derive(Deserialize, Debug)]
struct File {
    name: String,
}

fn filter_contains<'a>(
    months: &'a Vec<&str>,
    word: &String,
    blocklist: &HashSet<&str>,
) -> Vec<&'a str> {
    if word.is_empty() {
        return Vec::new();
    }

    months
        .iter()
        .filter(|m| m.contains(&word.to_lowercase()) && !blocklist.contains(*m)) // todo: maybe replace with fuzzy-rs
        .take(SHOW_LINES as usize)
        .cloned()
        // .map(|s| s.to_string())
        .collect::<Vec<&str>>()
    // .or_else(Vec::new)
}

fn render(
    arrow: u8,
    filtered_items: &Vec<&str>,
    chosen_items: &HashSet<&str>,
    stdout: &mut RawTerminal<std::io::Stdout>,
    typed: &String,
) {
    write!(
        stdout,
        "{}{}\rChosen items: {:?}\n",
        termion::cursor::Up(1),
        termion::clear::CurrentLine,
        chosen_items
    )
    .unwrap();
    write!(
        stdout,
        "{}\rEnter some input: {}{}\n\r",
        termion::clear::CurrentLine,
        typed,
        termion::cursor::Save
    )
    .unwrap();

    // render visible items
    let mut printed_lines = 0;
    for item in filtered_items {
        write!(
            stdout,
            "\r {} ",
            if printed_lines == arrow { '>' } else { ' ' }
        )
        .unwrap();
        write!(stdout, "{}{}\n", item, termion::clear::UntilNewline).unwrap();
        printed_lines += 1;
    }

    // print empty lines
    for _ in 0..(SHOW_LINES - printed_lines) {
        write!(stdout, "{}\n", termion::clear::CurrentLine).unwrap();
    }

    // print chosen
    write!(stdout, "\rDEBUG: {:?}\n", typed).unwrap();

    // restore
    write!(stdout, "{}", termion::cursor::Restore).unwrap();
    stdout.lock().flush().unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let request_url = format!("")
    let files =
        match minreq::get("https://api.github.com/repos/toptal/gitignore/contents/templates")
            .with_header("User-Agent", "git-ignore")
            .send()?
            .json::<Vec<File>>()
        {
            Ok(f) => f,
            Err(e) => return Err(e.into()),
        };

    let re = Regex::new(r"\.(patch|gitignore)").unwrap();
    let file_names: Vec<String> = files
        .iter()
        .map(|f| re.replace_all(&f.name, "").to_string())
        .collect();

    println!("files: {:?}", file_names);

    println!("Starting real CLI");

    let mut stdout = io::stdout()
        .into_raw_mode()
        .expect("Something went wrong switching into raw mode");
    let mut stdin = termion::async_stdin().keys();

    let months = vec![
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    let mut arrow: u8 = 0;
    let mut typed = String::new();

    let mut chosen_items: HashSet<&str> = HashSet::new();
    let mut filtered_items: Vec<&str> = Vec::new();

    render(arrow, &filtered_items, &chosen_items, &mut stdout, &typed);

    loop {
        let input = stdin.next();
        if let Some(Ok(key)) = input {
            match key {
                termion::event::Key::Up => {
                    if 0 < arrow {
                        arrow -= 1;
                    }
                }
                termion::event::Key::Down => {
                    if arrow < 2 {
                        arrow += 1;
                    }
                }
                termion::event::Key::Char('\t') => (),
                termion::event::Key::Char('\n') => {
                    if let Some(selected) = filtered_items.get(arrow as usize) {
                        chosen_items.insert(selected);
                        typed = String::new();
                    };
                    filtered_items = Vec::new();
                    arrow = 0;
                }
                termion::event::Key::Backspace => {
                    if typed.is_empty() {
                        continue; // todo: maybe exit loop
                    }
                    typed.pop();
                    filtered_items = filter_contains(&months, &typed, &chosen_items);
                }
                termion::event::Key::Char(character) => {
                    typed.push(character);
                    arrow = 0;
                    filtered_items = filter_contains(&months, &typed, &chosen_items);
                }
                _ => break,
            }
            render(arrow, &filtered_items, &chosen_items, &mut stdout, &typed);
        }
        thread::sleep(time::Duration::from_millis(50));
    }

    write!(stdout, "\n\r").unwrap();

    Ok(())
}
