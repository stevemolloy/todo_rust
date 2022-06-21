use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, Write};

use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode, KeyEvent},
    style::{style, Attribute, Color, PrintStyledContent, ResetColor, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand, Result,
};

enum Tab {
    TODO,
    DONE,
}

impl Tab {
    fn toggle(&self) -> Self {
        match self {
            Tab::TODO => Tab::DONE,
            Tab::DONE => Tab::TODO,
        }
    }
}

fn read_lines(filename: &str) -> Vec<String> {
    BufReader::new(File::open(filename).expect("no such file"))
        .lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

fn filter_and_strip<'a>(lines: &'a Vec<String>, prefix: &'a str) -> Vec<String> {
    lines
        .iter()
        .filter(|s| s.starts_with(prefix))
        .map(|s| s.strip_prefix(prefix).unwrap().to_string())
        .collect()
}

const PERSIST_FILE: &str = "/home/smolloy/.config/todo_rust/items";

fn main() -> Result<()> {
    let lines = read_lines(PERSIST_FILE);
    let mut todos: Vec<String> = filter_and_strip(&lines, "TODO: ");
    let mut dones: Vec<String> = filter_and_strip(&lines, "DONE: ");

    let mut curr_item = 0;
    let mut tab = Tab::TODO;

    let mut stdout = stdout();
    enable_raw_mode().unwrap();
    let mut fg_color: Color;
    let mut bg_color: Color;

    loop {
        stdout
            .execute(Clear(ClearType::All))?
            .execute(MoveTo(0, 0))?;

        let items;
        let title;
        let prefix;
        match tab {
            Tab::TODO => {
                title = "[TODO] DONE";
                items = todos.clone();
                prefix = " [ ] :: ";
            }
            Tab::DONE => {
                title = " TODO [DONE]";
                items = dones.clone();
                prefix = " [X] :: ";
            }
        }

        stdout.execute(PrintStyledContent(
            style(format!("{}\n\r", title)).attribute(Attribute::Bold),
        ))?;
        for (ind, item) in items.iter().enumerate() {
            if ind == curr_item {
                fg_color = Color::Black;
                bg_color = Color::White;
            } else {
                fg_color = Color::White;
                bg_color = Color::Black;
            }
            stdout.execute(PrintStyledContent(
                style(format!("{}{}\n\r", prefix, item))
                    .with(fg_color)
                    .on(bg_color),
            ))?;
        }

        match read().unwrap() {
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                ..
            }) => {
                stdout
                    .execute(Clear(ClearType::All))?
                    .execute(MoveTo(0, 0))?
                    .execute(PrintStyledContent(
                        style(format!("Add a todo here:\n\r")).attribute(Attribute::Bold),
                    ))?;
                disable_raw_mode()?;
                let mut blah = String::new();
                match stdin().read_line(&mut blah) {
                    Ok(_) => todos.push(blah.trim_end().to_string()),
                    Err(error) => println!("error: {error}"),
                }
                enable_raw_mode()?;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => {
                if let Ok(mut output) = File::create(PERSIST_FILE) {
                    for item in todos {
                        writeln!(output, "TODO: {}\n", item).unwrap();
                    }
                    for item in dones {
                        writeln!(output, "DONE: {}\n", item).unwrap();
                    }
                } else {
                    stdout.execute(PrintStyledContent(style("FAIL")))?;
                }
                break;
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => {
                if curr_item < items.len() - 1 {
                    curr_item += 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) => {
                if curr_item > 0 {
                    curr_item -= 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Tab, ..
            }) => {
                curr_item = 0;
                tab = tab.toggle();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => match tab {
                Tab::TODO => {
                    dones.push(todos.remove(curr_item));
                }
                Tab::DONE => {
                    todos.push(dones.remove(curr_item));
                }
            },
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                ..
            }) => match tab {
                Tab::TODO => {
                    todos.remove(curr_item);
                }
                Tab::DONE => {
                    dones.remove(curr_item);
                }
            },
            _ => (),
        }
    }

    disable_raw_mode().unwrap();
    stdout.execute(ResetColor)?;

    Ok(())
}
