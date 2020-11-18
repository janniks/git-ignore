use std::collections::HashSet;
use std::error::Error;
use std::io::{self, Write};
use std::thread;
use std::time;

use regex::Regex;

use serde::Deserialize;
use sublime_fuzzy::best_match;
use termion;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;

const SHOW_LINES: u8 = 4;
const TEMPLATES_URL: &str = "https://api.github.com/repos/toptal/gitignore/contents/templates";

#[derive(Deserialize, Debug)]
struct File {
    name: String,
}

#[derive(Debug)]
struct Item<'a> {
    name: &'a String,
    score: isize,
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
    arrow: u8,
    filtered_items: &Vec<&String>,
    chosen_items: &HashSet<&String>,
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

fn main() -> Result<(), Box<dyn Error>> {
    println!("Loading templates from GitHub");
    let ignores = get_ignores().expect(" ! Unable to load templates from GitHub");

    let mut stdout = io::stdout()
        .into_raw_mode()
        .expect(" ! Unable into raw mode");
    let mut stdin = termion::async_stdin().keys();

    let mut arrow: u8 = 0;
    let mut typed = String::new();

    let mut chosen_items: HashSet<&String> = HashSet::new();
    let mut filtered_items: Vec<&String> = Vec::new();

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
                    if arrow < SHOW_LINES - 1 {
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
                    filtered_items = filter_fuzzy(&ignores, &typed, &chosen_items);
                }
                termion::event::Key::Char(character) => {
                    typed.push(character);
                    arrow = 0;
                    filtered_items = filter_fuzzy(&ignores, &typed, &chosen_items);
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
