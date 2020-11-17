use std::io::{self, Write};
use std::thread;
use std::time;

use termion;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;

const SHOW_LINES: u8 = 3;

fn filter_contains<'a>(months: &'a Vec<&str>, word: &String) -> Vec<&'a str> {
    if word.is_empty() {
        return Vec::new();
    }

    months
        .iter()
        .filter(|m| m.contains(&word.to_lowercase())) // todo: replace with fuzzy-rs
        .take(SHOW_LINES as usize)
        .cloned()
        // .map(|s| s.to_string())
        .collect::<Vec<&str>>()
    // .or_else(Vec::new)
}

fn render(
    arrow: u8,
    filtered_items: &Vec<&str>,
    chosen_items: &Vec<&str>,
    stdout: &mut RawTerminal<std::io::Stdout>,
    typed: &String,
) {
    write!(stdout, "Enter some input: {}\n\r").unwrap();
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
        write!(stdout, "{}{}\n", item, termion::clear::UntilNewline).unwrap();
        printed_lines += 1;
    }

    // print empty lines
    for _ in 0..(SHOW_LINES - printed_lines) {
        write!(stdout, "{}\n", termion::clear::CurrentLine).unwrap();
    }

    // print chosen
    write!(stdout, "\rChosen items: {:?}\n", chosen_items).unwrap();

    write!(stdout, "\rTyped: {:?}\n", typed).unwrap();

    // restore
    write!(stdout, "{}", termion::cursor::Restore).unwrap();
    stdout.lock().flush().unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let mut chosen_items: Vec<&str> = Vec::new();
    let mut filtered_items: Vec<&str> = Vec::new();

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
                termion::event::Key::Char('\n') | termion::event::Key::Char('\r') => {
                    let selected = match filtered_items.get(arrow as usize) {
                        Some(s) => s,
                        None => continue,
                    };

                    chosen_items.push(selected);
                    typed = String::new();
                    arrow = 0;
                }
                termion::event::Key::Backspace => {
                    if typed.is_empty() {
                        continue; // maybe exit loop?
                    }
                    typed.pop();
                    filtered_items = filter_contains(&months, &typed);
                    write!(
                        stdout,
                        "{}{}{}",
                        termion::cursor::Left(1),
                        termion::clear::UntilNewline,
                        termion::cursor::Save
                    )
                    .unwrap();
                }
                termion::event::Key::Char(character) => {
                    typed.push(character);
                    arrow = 0;
                    filtered_items = filter_contains(&months, &typed);
                    write!(stdout, "{}{}", character, termion::cursor::Save).unwrap();
                }
                _ => break,
            }
            render(arrow, &filtered_items, &chosen_items, &mut stdout, &typed)
        }
        thread::sleep(time::Duration::from_millis(50));
    }

    // write!(stdout, "\n\rThe months are: {}", words).unwrap();
    write!(stdout, "\n\r").unwrap();

    Ok(())
}
