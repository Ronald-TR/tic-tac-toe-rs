use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, sync::Arc, thread, time::Duration};
use tokio::runtime::Handle;
use tokio::sync::{Mutex, MutexGuard};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;
pub mod utils;

#[macro_use]
extern crate lazy_static;

enum InputMode {
    Popup,
    Editing,
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// room id to share and sync the board
    room_id: String,
    /// player current move
    command: String,
    /// player shape
    shape: String,
    /// winner announcement returned by the server
    winner_message: String,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Editing,
            room_id: "".to_string(),
            command: "".to_string(),
            shape: "".to_string(),
            winner_message: "".to_string(),
        }
    }
}

lazy_static! {
    static ref BOARD: Arc<Mutex<String>> = {
        let m = String::new();
        Arc::new(Mutex::new(m))
    };
    static ref APP: Arc<Mutex<App>> = {
        let app = App::default();
        Arc::new(Mutex::new(app))
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    utils::create_or_join_room().await?;

    let handle = Arc::new(Handle::current());

    thread::spawn(move || {
        let _ = main_term(&handle.clone());
    })
    .join()
    .expect("term thread panicked");
    Ok(())
}

fn main_term(handle: &Arc<Handle>) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = APP.blocking_lock();
    let res = run_app(&mut terminal, app, &handle);

    // restore terminal
    disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();
    terminal.show_cursor().unwrap();

    if let Err(err) = res {
        println!("{:?}", err)
    }
    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: MutexGuard<App>,
    handle: &Handle,
) -> io::Result<()> {
    loop {
        let room_id = app.room_id.clone();
        let future = handle.spawn(async {
            let response = utils::loop_board_state(room_id).await.unwrap();
            response
        });
        let response = handle.block_on(future);
        match response {
            Ok(x) => {
                if x.has_winner {
                    app.input_mode = InputMode::Popup;
                    app.winner_message = format!(
                        "We have a winner! {} is the winner",
                        x.shape_winner
                    );
                }
            },
            Err(_) => continue,
        };

        terminal.draw(|f| ui(f, &app))?;

        // this event::poll prevents the main thread being blocked by event::read
        if let Ok(evt) = event::poll(Duration::from_millis(100)) {
            if evt == false {
                continue;
            }
        };
        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Popup => match key.code {
                    KeyCode::Esc => {
                        return Ok(());
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        app.command = app.input.drain(..).collect();

                        let room_id = app.room_id.clone();
                        let command = app.command.clone();
                        let shape = app.shape.clone();
                        let future = handle.spawn(async {
                            utils::do_movement(room_id, command, shape).await.unwrap()
                        });
                        //
                        let result = handle.block_on(future).unwrap();
                        if result.contains("winner") {
                            app.winner_message = result;
                            app.input_mode = InputMode::Popup;
                        }
                    }
                    KeyCode::Char(c) => {
                        let offset = 3;
                        // mask the input to the format: "<X><Whitespace><Y>"
                        if (c.is_numeric()) && app.input.len() < offset {
                            app.input.push(c);
                            app.input.push(' ');
                        }
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        return Ok(());
                    }
                    _ => {}
                },
            }
        }
    }
}

fn build_layout(area: Rect)-> Vec<Rect> {
    Layout::default()
    .direction(Direction::Vertical)
    .margin(2)
    .constraints(
        [
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(1),
        ]
        .as_ref(),
    )
    .split(area)
}

fn draw_help_navbar<B: Backend>(f: &mut Frame<B>, app: &App, chunks: &Vec<Rect>) {
    let (msg, style) = (
        vec![
            Span::raw("Enter your move as a chordinate! ex: "),
            Span::styled("0 0 or 1 1", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" marks your move on the board, "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" to send the move"),
        ],
        Style::default(),
    );
    let mut text = Text::from(Spans::from(msg));
    text.extend(Text::raw(format!("room id: {}", app.room_id)));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = build_layout(f.size());
    draw_help_navbar(f, app, &chunks);

    let input = Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Popup => {
            let block = Block::default()
                .title("Congrats")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::LightGreen).fg(Color::Black));
            let area = centered_rect(45, 20, f.size());
            let content_area = centered_rect(40, 10, f.size());
            let content = Paragraph::new(Text::raw(format!(
                "{}\n{}",
                app.winner_message, "press Esc to exit"
            )))
            .style(Style::default().fg(Color::DarkGray));
            f.render_widget(Clear, area); //this clears out the background
            f.render_widget(block, area);
            f.render_widget(content, content_area);
        }

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }
    let board = BOARD.blocking_lock();

    let text = Text::raw(board.as_str());
    let message = Paragraph::new(text)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(message, chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
