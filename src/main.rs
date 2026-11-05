use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write, stdout};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use clap::Parser;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::widgets::{Borders, Padding};
use ratatui::{DefaultTerminal, TerminalOptions, Viewport};
use ratatui_textarea::{CursorMove, Input, Key, TextArea};

use crate::args::ArgsFullscreenMode;

mod args;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let args = args::Args::parse();

    let textarea = match File::open(&args.file) {
        Ok(file) => {
            let mut textarea = BufReader::new(file)
                .lines()
                .collect::<io::Result<TextArea>>()?;
            textarea.move_cursor(CursorMove::Bottom);
            textarea.move_cursor(CursorMove::End);
            textarea.insert_newline();
            textarea.move_cursor(CursorMove::Top);
            textarea
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => TextArea::default(),
        Err(err) => return Err(err.into()),
    };

    let total_height = args.height
        + if args.title.is_some() { 1 } else { 0 }
        + if !args.no_keybinds { 2 } else { 0 };

    let fullscreen = match args.fullscreen {
        ArgsFullscreenMode::Disabled => false,
        ArgsFullscreenMode::Enabled => true,
        ArgsFullscreenMode::Adaptive(threshold) => {
            textarea.lines().iter().count() > threshold.into()
        }
    };

    let mut terminal = ratatui::init_with_options(TerminalOptions {
        viewport: if fullscreen {
            Viewport::Fullscreen
        } else {
            Viewport::Inline(total_height)
        },
    });
    terminal.clear()?;
    let cursor_pos = terminal.get_cursor_position()?;
    if fullscreen {
        execute!(stdout(), EnterAlternateScreen)?;
    }

    run(textarea, &mut terminal, fullscreen, &args)?;
    ratatui::restore();

    if fullscreen {
        execute!(stdout(), LeaveAlternateScreen)?;
    } else {
        terminal.set_cursor_position(cursor_pos)?;
        println!("\n");
    }

    Ok(())
}

fn run(
    mut textarea: TextArea,
    terminal: &mut DefaultTerminal,
    fullscreen: bool,
    args: &args::Args,
) -> color_eyre::Result<()> {
    textarea.set_block(
        Block::new()
            .borders(Borders::LEFT)
            .border_style(Style::new().gray())
            .border_set(border::THICK)
            .padding(Padding::horizontal(1)),
    );

    let title = args
        .title
        .as_ref()
        .map(|title| Line::from(title.as_str().dark_gray()));
    let help = Line::from(vec![
        "shift+enter ".dark_gray(),
        "insert newline ".dark_gray().dim(),
        "• ".dark_gray().dim(),
        "ctrl+e ".dark_gray(),
        "open editor ".dark_gray().dim(),
        "• ".dark_gray().dim(),
        "esc ".dark_gray(),
        "abort ".dark_gray(),
        "• ".dark_gray().dim(),
        "enter ".dark_gray(),
        "save ".dark_gray().dim(),
    ]);

    let mut layout = if fullscreen {
        vec![Constraint::Fill(1)]
    } else {
        vec![Constraint::Length(args.height)]
    };
    if title.is_some() {
        layout.insert(0, Constraint::Length(1));
    }
    if !args.no_keybinds {
        layout.extend([Constraint::Length(1), Constraint::Length(1)]);
    }
    let layout = Layout::vertical(layout);

    loop {
        terminal.draw(|f| {
            let mut areas = f.area().layout_vec(&layout).into_iter();

            if title.is_some() {
                title.clone().render(areas.next().unwrap(), f.buffer_mut());
            }
            textarea.render(areas.next().unwrap(), f.buffer_mut());
            if !args.no_keybinds {
                help.clone().render(areas.nth(1).unwrap(), f.buffer_mut());
            }
        })?;

        match crossterm::event::read()?.into() {
            Input {
                key: Key::Char('j'),
                ctrl: true,
                shift: false,
                alt: false,
            } => textarea.insert_newline(),
            Input {
                key: Key::Char('e'),
                ctrl: true,
                shift: false,
                alt: false,
            } => {
                #[cfg(not(target_os = "windows"))]
                let Ok(editor) = std::env::var("EDITOR") else {
                    eprintln!("$EDITOR not set");
                    continue;
                };
                #[cfg(target_os = "windows")]
                let editor = std::env::var("EDITOR").unwrap_or_else(|_| "notepad".to_string());

                terminal.draw(|f| {
                    let mut areas = f.area().layout_vec(&layout).into_iter();

                    if title.is_some() {
                        title.clone().render(areas.next().unwrap(), f.buffer_mut());
                    }
                    Paragraph::new("File opened in editor...".dark_gray())
                        .block(
                            Block::new()
                                .borders(Borders::LEFT)
                                .border_style(Style::new().gray())
                                .border_set(border::THICK)
                                .padding(Padding::horizontal(1)),
                        )
                        .render(areas.next().unwrap(), f.buffer_mut());
                    if !args.no_keybinds {
                        help.clone().render(areas.nth(1).unwrap(), f.buffer_mut());
                    }
                })?;

                save_file(textarea, args)?;
                let mut cmd = Command::new(editor).arg(&args.file).spawn()?;
                while cmd.try_wait()?.is_none() {
                    sleep(Duration::from_millis(300));
                }
                return Ok(());
            }
            Input { key: Key::Esc, .. }
            | Input {
                key: Key::Char('c'),
                ctrl: true,
                shift: false,
                alt: false,
            } => return Ok(()),
            Input {
                key: Key::Enter, ..
            } => {
                save_file(textarea, args)?;
                return Ok(());
            }
            event => {
                textarea.input(event);

                if textarea.cursor().0 == textarea.lines().len() - 1 {
                    let line = textarea.lines().last().unwrap();
                    if textarea.cursor().1 == line.len() && !line.is_empty() {
                        textarea.insert_newline();
                        textarea.move_cursor(CursorMove::Back);
                    }
                }
            }
        }
    }
}

fn save_file(textarea: TextArea, args: &args::Args) -> color_eyre::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args.file)?;
    file.write_all(textarea.into_lines().join("\n").as_bytes())?;
    file.flush()?;
    Ok(())
}
