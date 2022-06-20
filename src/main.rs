use std::fs::File;
use std::io::{stdout, BufRead, BufReader};

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

fn main() -> Result<()> {
    let lines: Vec<String> =
        BufReader::new(File::open("/home/smolloy/.config/todo_rust/items").expect("no such file"))
            .lines()
            .map(|l| l.expect("Could not parse line"))
            .collect();

    let mut todos: Vec<&str> = lines
        .iter()
        .filter(|s| s.starts_with("TODO: "))
        .map(|s| s.strip_prefix("TODO: ").unwrap() as &str)
        .collect();
    let mut dones: Vec<&str> = lines
        .iter()
        .filter(|s| s.starts_with("DONE: "))
        .map(|s| s.strip_prefix("DONE: ").unwrap() as &str)
        .collect();
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
                code: KeyCode::Char('q'),
                ..
            }) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            }) => {
                if curr_item < items.len() - 1 {
                    curr_item += 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up, ..
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
