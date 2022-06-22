use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::Event::Key,
    event::KeyCode::{Char, Down, Enter, Tab, Up},
    event::{read, KeyEvent},
    style::{style, Attribute, Color, PrintStyledContent, ResetColor, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand, Result,
};

enum UiTab {
    TODO,
    DONE,
}

impl UiTab {
    fn toggle(&self) -> Self {
        match self {
            UiTab::TODO => UiTab::DONE,
            UiTab::DONE => UiTab::TODO,
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

fn move_up(mut curr: usize) -> usize {
    if curr > 0 {
        curr -= 1;
    }
    return curr;
}

fn move_down(mut curr: usize, lim: usize) -> usize {
    if curr < lim - 1 {
        curr += 1;
    }
    return curr;
}

const PERSIST_FILE: &str = "/home/smolloy/.config/todo_rust/items";

fn main() -> Result<()> {
    let lines = read_lines(PERSIST_FILE);
    let mut todos: Vec<String> = filter_and_strip(&lines, "TODO: ");
    let mut dones: Vec<String> = filter_and_strip(&lines, "DONE: ");

    let mut curr_item = 0;
    let mut tab = UiTab::TODO;

    let mut stdout = stdout();
    enable_raw_mode().unwrap();
    let mut fg_color: Color;
    let mut bg_color: Color;

    loop {
        stdout
            .execute(Hide)?
            .execute(Clear(ClearType::All))?
            .execute(MoveTo(0, 0))?;

        let items;
        let title;
        let prefix;
        match tab {
            UiTab::TODO => {
                title = "[TODO] DONE";
                items = todos.clone();
                prefix = " [ ] :: ";
            }
            UiTab::DONE => {
                title = " TODO [DONE]";
                items = dones.clone();
                prefix = " [X] :: ";
            }
        }
        if curr_item >= items.len() {
            curr_item = items.len() - 1;
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
            Key(KeyEvent {
                code: Char('a'), ..
            }) => {
                stdout
                    .execute(Show)?
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
                stdout.execute(Hide)?;
            }
            Key(KeyEvent {
                code: Char('q'), ..
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
            Key(KeyEvent { code: Down, .. })
            | Key(KeyEvent {
                code: Char('j'), ..
            }) => curr_item = move_down(curr_item, items.len()),
            Key(KeyEvent { code: Up, .. })
            | Key(KeyEvent {
                code: Char('k'), ..
            }) => curr_item = move_up(curr_item),
            Key(KeyEvent {
                code: Char('J'), ..
            }) => {
                if curr_item < items.len() - 1 {
                    match tab {
                        UiTab::TODO => todos.swap(curr_item, curr_item + 1),
                        UiTab::DONE => dones.swap(curr_item, curr_item + 1),
                    }
                    curr_item += 1;
                }
            }
            Key(KeyEvent {
                code: Char('K'), ..
            }) => {
                if curr_item < items.len() && curr_item > 0 {
                    match tab {
                        UiTab::TODO => todos.swap(curr_item, curr_item - 1),
                        UiTab::DONE => dones.swap(curr_item, curr_item - 1),
                    }
                    curr_item -= 1;
                }
            }
            Key(KeyEvent { code: Tab, .. }) => {
                curr_item = 0;
                tab = tab.toggle();
            }
            Key(KeyEvent { code: Enter, .. }) => match tab {
                UiTab::TODO => {
                    dones.push(todos.remove(curr_item));
                }
                UiTab::DONE => {
                    todos.push(dones.remove(curr_item));
                }
            },
            Key(KeyEvent {
                code: Char('d'), ..
            }) => match tab {
                UiTab::TODO => {
                    todos.remove(curr_item);
                }
                UiTab::DONE => {
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
