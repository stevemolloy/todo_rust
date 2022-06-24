use std::collections::VecDeque;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::Event::Key,
    event::KeyCode::{BackTab, Char, Down, Enter, Tab, Up},
    event::{read, KeyEvent},
    style::{style, Attribute, Color, PrintStyledContent, ResetColor, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand, Result,
};

enum UiTab {
    TODO,
    DONE,
    ARCHIVED,
}

impl UiTab {
    fn rotate(&self) -> Self {
        match self {
            UiTab::TODO => UiTab::DONE,
            UiTab::DONE => UiTab::ARCHIVED,
            UiTab::ARCHIVED => UiTab::TODO,
        }
    }

    fn rotate_back(&self) -> Self {
        match self {
            UiTab::TODO => UiTab::ARCHIVED,
            UiTab::DONE => UiTab::TODO,
            UiTab::ARCHIVED => UiTab::DONE,
        }
    }
}

fn read_lines(filename: &str) -> Vec<String> {
    BufReader::new(File::open(filename).expect("no such file"))
        .lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

fn filter_and_strip<'a>(lines: &'a Vec<String>, prefix: &'a str) -> VecDeque<String> {
    lines
        .iter()
        .filter(|s| s.starts_with(prefix))
        .map(|s| s.strip_prefix(prefix).unwrap().to_string())
        .collect()
}

fn move_up(mut curr: usize, lim: usize) -> usize {
    if curr > 0 {
        curr -= 1;
    } else {
        curr = lim - 1;
    }
    return curr;
}

fn move_down(mut curr: usize, lim: usize) -> usize {
    if lim == 0 {
        return curr;
    }
    if curr < lim - 1 {
        curr += 1;
    } else {
        curr = 0;
    }
    return curr;
}

fn save(todo_list: VecDeque<String>, done_list: VecDeque<String>) {
    if let Ok(mut output) = File::create(PERSIST_FILE) {
        for item in todo_list {
            writeln!(output, "TODO: {}\n", item).unwrap();
        }
        for item in done_list {
            writeln!(output, "DONE: {}\n", item).unwrap();
        }
    } else {
        stdout().execute(PrintStyledContent(style("FAIL"))).unwrap();
    }
}

const PERSIST_FILE: &str = "/home/smolloy/.config/todo_rust/items";

fn main() -> Result<()> {
    let lines = read_lines(PERSIST_FILE);
    let mut todos: VecDeque<String> = filter_and_strip(&lines, "TODO: ");
    let mut dones: VecDeque<String> = filter_and_strip(&lines, "DONE: ");
    let mut archiveds: VecDeque<String> = filter_and_strip(&lines, "ARCHIVED: ");

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
                title = "[TODO] DONE  ARCHIVED ";
                items = todos.clone();
                prefix = " [ ] :: ";
            }
            UiTab::DONE => {
                title = " TODO [DONE] ARCHIVED ";
                items = dones.clone();
                prefix = " [X] :: ";
            }
            UiTab::ARCHIVED => {
                title = " TODO  DONE [ARCHIVED]";
                items = archiveds.clone();
                prefix = " --- :: ";
            }
        }
        if curr_item >= items.len() && curr_item != 0 {
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
                    Ok(_) => todos.push_front(blah.trim_end().to_string()),
                    Err(error) => println!("error: {error}"),
                }
                enable_raw_mode()?;
                stdout.execute(Hide)?;
            }
            Key(KeyEvent {
                code: Char('q'), ..
            }) => {
                save(todos.clone(), dones.clone());
                break;
            }
            Key(KeyEvent {
                code: Char('w'), ..
            }) => save(todos.clone(), dones.clone()),
            Key(KeyEvent { code: Down, .. })
            | Key(KeyEvent {
                code: Char('j'), ..
            }) => curr_item = move_down(curr_item, items.len()),
            Key(KeyEvent { code: Up, .. })
            | Key(KeyEvent {
                code: Char('k'), ..
            }) => curr_item = move_up(curr_item, items.len()),
            Key(KeyEvent {
                code: Char('J'), ..
            }) => {
                if curr_item < items.len() - 1 {
                    match tab {
                        UiTab::TODO => todos.swap(curr_item, curr_item + 1),
                        UiTab::DONE => dones.swap(curr_item, curr_item + 1),
                        UiTab::ARCHIVED => archiveds.swap(curr_item, curr_item + 1),
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
                        UiTab::ARCHIVED => archiveds.swap(curr_item, curr_item - 1),
                    }
                    curr_item -= 1;
                }
            }
            Key(KeyEvent { code: Tab, .. }) => {
                curr_item = 0;
                tab = tab.rotate();
            }
            Key(KeyEvent { code: BackTab, .. }) => {
                curr_item = 0;
                tab = tab.rotate_back();
            }
            Key(KeyEvent { code: Enter, .. }) => match tab {
                UiTab::TODO => match todos.remove(curr_item) {
                    Some(n) => dones.push_front(n),
                    None => (),
                },
                UiTab::DONE => match dones.remove(curr_item) {
                    Some(n) => todos.push_front(n),
                    None => (),
                },
                UiTab::ARCHIVED => (),
            },
            Key(KeyEvent {
                code: Char('d'), ..
            }) => match tab {
                UiTab::TODO => match todos.remove(curr_item) {
                    Some(n) => archiveds.push_front(n),
                    None => (),
                },
                UiTab::DONE => match dones.remove(curr_item) {
                    Some(n) => archiveds.push_front(n),
                    None => (),
                },
                UiTab::ARCHIVED => match archiveds.remove(curr_item) {
                    Some(n) => todos.push_front(n),
                    None => (),
                },
            },
            _ => (),
        }
    }

    disable_raw_mode().unwrap();
    stdout.execute(ResetColor)?;

    Ok(())
}
