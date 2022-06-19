use std::io::stdout;

use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode, KeyEvent},
    style::{style, Color, PrintStyledContent, ResetColor, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand, Result,
};

enum Tab {
    TODO,
    DONE,
}

fn main() -> Result<()> {
    let mut std = stdout();
    enable_raw_mode().unwrap();

    let todos = vec!["Bake bread", "Buy things", "Write a TODO app"];
    let dones = vec![
        "Run 5 km",
        "Email FemtoMAX",
        "Check downtimes",
        "Figure out the thing",
    ];
    let mut curr_item = 0;

    let mut fg_color: Color;
    let mut bg_color: Color;

    let mut tab = Tab::TODO;

    loop {
        std.execute(Clear(ClearType::All))?.execute(MoveTo(0, 0))?;

        let items;
        let title;
        match tab {
            Tab::TODO => {
                title = "[TODO] DONE";
                items = &todos;
            }
            Tab::DONE => {
                title = " TODO [DONE]";
                items = &dones;
            }
        }

        std.execute(PrintStyledContent(style(format!("{}\n\r", title))))?;
        for (ind, todo) in items.iter().enumerate() {
            if ind == curr_item {
                fg_color = Color::Black;
                bg_color = Color::White;
            } else {
                fg_color = Color::White;
                bg_color = Color::Black;
            }
            std.execute(PrintStyledContent(
                style(format!("{}\n\r", todo)).with(fg_color).on(bg_color),
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
                match tab {
                    Tab::DONE => tab = Tab::TODO,
                    Tab::TODO => tab = Tab::DONE,
                }
            }
            _ => (),
        }
    }

    disable_raw_mode().unwrap();
    std.execute(ResetColor)?;
    Ok(())
}
