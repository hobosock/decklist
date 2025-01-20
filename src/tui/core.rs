use std::{
    io::{self, stdout, Stdout},
    thread,
};

use async_std::task;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, Paragraph, Tabs,
    },
    Frame, Terminal,
};
use ratatui_explorer::FileExplorer;

use crate::{
    app::App,
    startup::{startup_checks, StartupChecks},
};

/// a type alias for terminal type used
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// tabs for main TUI interface
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum MenuTabs {
    #[default]
    Welcome,
    Collection,
    Deck,
    Help,
    Debug,
}

/// initialize the terminal
pub fn init() -> io::Result<Tui> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

/// restore the terminal to it's original state
pub fn restore() -> io::Result<()> {
    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// main UI definition
pub fn ui(
    frame: &mut Frame,
    app: &mut App,
    explorer: &mut FileExplorer,
    explorer2: &mut FileExplorer,
) {
    // start new thread to run start up processes
    if !app.startup {
        let startup_channel = app.startup_channel.0.clone();
        thread::spawn(move || {
            let startup_results = task::block_on(startup_checks());
            match startup_channel.send(startup_results) {
                Ok(()) => {}
                Err(_) => {}
            }
        });
        match app.startup_channel.1.try_recv() {
            Ok(startup_checks) => {
                // TODO: just ad a StartupChecks struct to app struct, pass in one line
                app.startup = true;
                app.config_exist = startup_checks.config_exists;
                app.database_exist = startup_checks.database_exists;
                app.collection_exist = startup_checks.collection_exists;
                app.directory_exist = startup_checks.directory_exists;
                app.database_status = startup_checks.database_status;
                app.directory_status = startup_checks.directory_status;
                app.config_status = startup_checks.config_status;
                app.collection_status = startup_checks.collection_status;
            }
            Err(_) => {}
        }
    }
    // split area into 3 chunks (tabs/main/keys)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(2),
            Constraint::Length(3),
        ])
        .split(frame.area());

    // tabs for switching between menus
    let tabs = Tabs::new(vec!["1. Welcome", "2. Collection", "3. Deck", "4. Help"])
        .block(Block::default().title("| Menu |").borders(Borders::ALL))
        .style(Style::default().white())
        .highlight_style(Style::default().cyan().bold())
        .select(app.active_tab as usize);

    // define main/center area for display
    let version = Title::from(
        Line::from(vec!["| Deck Checker v0.1.0 |".into()]).style(Style::default().cyan().bold()),
    )
    .alignment(Alignment::Center)
    .position(Position::Bottom);
    let main_block = Block::default()
        .title(version)
        .borders(Borders::ALL)
        .border_set(border::THICK);

    // change bottom two chunks based on selected tab
    let instructions_block = Block::default().borders(Borders::ALL);
    let mut instructions_text = Text::from(vec![Line::from(vec!["test".into()])]);
    match app.active_tab {
        MenuTabs::Welcome => {
            let mut instructions_text = Text::from(vec![Line::from(vec![
                "<Q>".yellow().bold(),
                " Quit ".into(),
                "<C>".yellow().bold(),
                " Load Collection ".into(),
                "<D>".yellow().bold(),
                " Download Database ".into(),
            ])]);
            draw_welcome_main(app, frame, chunks[1], main_block);
        }
        _ => {}
    }

    let instructions = Paragraph::new(instructions_text)
        .centered()
        .block(instructions_block);

    // render
    frame.render_widget(tabs, chunks[0]);
    frame.render_widget(instructions, chunks[2]);
}

/// draw the main window on the welcome tab
fn draw_welcome_main(app: &mut App, frame: &mut Frame, chunk: Rect, main_block: Block) {
    // draw startup check status
    let directory_status_line = if app.directory_exist {
        app.directory_status.clone().green()
    } else {
        app.directory_status.clone().red()
    };
    let config_status_line = if app.config_exist {
        app.config_status.clone().green()
    } else {
        app.config_status.clone().red()
    };
    let database_status_line = if app.database_exist {
        app.database_status.clone().green()
    } else {
        app.database_status.clone().red()
    };
    let collection_status_line = if app.collection_exist {
        app.collection_status.clone().green()
    } else {
        app.collection_status.clone().red()
    };
    let directory_line = Line::from(vec!["Directory: ".into(), directory_status_line]);
    let config_line = Line::from(vec!["Config file: ".into(), config_status_line]);
    let database_line = Line::from(vec!["Database: ".into(), database_status_line]);
    let collection_line = Line::from(vec!["Collection: ".into(), collection_status_line]);
    let status_paragraph = Paragraph::new(Text::from(vec![
        directory_line,
        config_line,
        database_line,
        collection_line,
    ]))
    .centered()
    .block(main_block);
    frame.render_widget(status_paragraph, chunk);
}
