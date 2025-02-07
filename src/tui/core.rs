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
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    prelude::{CrosstermBackend, Widget},
    style::{Style, Stylize},
    symbols::{border, scrollbar},
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, Tabs, Wrap,
    },
    Frame, Terminal,
};
use ratatui_explorer::FileExplorer;

use crate::{app::App, startup::startup_checks};

/// a type alias for terminal type used
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// tabs for main TUI interface
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum MenuTabs {
    #[default]
    Welcome,
    Database,
    Collection,
    Deck,
    Missing,
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
                // TODO: just add a StartupChecks struct to app struct, pass in one line
                app.startup = true;
                app.config_exist = startup_checks.config_exists;
                app.database_exist = startup_checks.database_exists;
                app.collection_exist = startup_checks.collection_exists;
                app.directory_exist = startup_checks.directory_exists;
                app.data_directory_exist = startup_checks.data_directory_exists;
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
    let tabs = Tabs::new(vec![
        "1. Welcome",
        "2. Database",
        "3. Collection",
        "4. Deck",
        "5. Missing",
        "6. Help",
    ])
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
            instructions_text = Text::from(vec![Line::from(vec![
                "<Q>".yellow().bold(),
                " Quit ".into(),
                "<C>".yellow().bold(),
                " Load Collection ".into(),
                "<D>".yellow().bold(),
                " Download Database ".into(),
            ])]);
            draw_welcome_main(app, frame, chunks[1], main_block);
        }
        MenuTabs::Collection => {
            instructions_text = Text::from(vec![Line::from(vec![
                "<Q>".yellow().bold(),
                " Quit ".into(),
                "<S>".yellow().bold(),
                " Load file ".into(),
                "<Esc>".yellow().bold(),
                " Reset file ".into(),
                "<Up/Down>".yellow().bold(),
                " Navigate ".into(),
                "<Left/Backspace>".yellow().bold(),
                " Exit Directory ".into(),
                "<Right/Enter>".yellow().bold(),
                " Down Directory ".into(),
            ])]);
            draw_collection_main(app, frame, chunks[1], main_block, explorer);
        }
        MenuTabs::Deck => {
            instructions_text = Text::from(vec![Line::from(vec![
                "<Q>".yellow().bold(),
                " Quit ".into(),
                "<S>".yellow().bold(),
                " Load file ".into(),
                "<Esc>".yellow().bold(),
                " Reset file ".into(),
                "<Up/Down>".yellow().bold(),
                " Navigate ".into(),
                "<Left/Backspace>".yellow().bold(),
                " Exit Directory ".into(),
                "<Right/Enter>".yellow().bold(),
                " Down Directory ".into(),
            ])]);
            draw_decklist_main(app, frame, chunks[1], main_block, explorer2);
        }
        MenuTabs::Missing => {
            instructions_text = Text::from(vec![Line::from(vec![
                "<Q>".yellow().bold(),
                " Quit ".into(),
                "<C>".yellow().bold(),
                " Copy to clipboard ".into(),
                "<S>".yellow().bold(),
                " Save to file ".into(),
                "<Up/Down>".yellow().bold(),
                " Navigate ".into(),
            ])]);
            draw_missing_main(app, frame, chunks[1], main_block);
        }
        MenuTabs::Debug => {
            draw_debug_main(app, frame, chunks[1], main_block);
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

fn draw_debug_main(app: &mut App, frame: &mut Frame, chunk: Rect, main_block: Block) {
    let debug_text = Paragraph::new(Text::from(app.debug_string.clone()))
        .wrap(Wrap { trim: true })
        .block(main_block);
    frame.render_widget(debug_text, chunk);
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
    .wrap(Wrap { trim: true })
    .centered()
    .block(main_block);
    frame.render_widget(status_paragraph, chunk);
}

/// draws the main block of the Collection tab
fn draw_collection_main(
    app: &mut App,
    frame: &mut Frame,
    chunk: Rect,
    main_block: Block,
    explorer: &mut FileExplorer,
) {
    // split into two sections - small one for info text and main for displaying file explorer
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(2)])
        .split(main_block.inner(chunk));
    if app.collection.is_none() {
        // TODO: cleanup dead branch
        // app.collection_status = "Please select a collection file from Moxfield.".to_string();
    } else {
        app.collection_status = format!(
            "Collection loaded successfully.  Using {}",
            app.collection_file_name.as_ref().unwrap() // NOTE: should exist if you get to this branch
        );
    }
    let file_paragraph = Paragraph::new(app.collection_status.clone()).wrap(Wrap { trim: true });
    frame.render_widget(main_block, chunk);
    frame.render_widget(file_paragraph, sections[0]);
    if app.collection.is_some() {
        let mut lines: Vec<Line> = Vec::new();
        for card in app.collection.as_ref().unwrap() {
            lines.push(Line::from(format!("{}", card)));
        }
        app.collection_scroll_state = app.collection_scroll_state.content_length(lines.len());
        let collection_paragraph = Paragraph::new(lines[app.collection_scroll..].to_vec());
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));
        frame.render_widget(collection_paragraph, sections[1]);
        frame.render_stateful_widget(
            scrollbar,
            sections[1].inner(Margin {
                vertical: 0,
                horizontal: 1,
            }),
            &mut app.collection_scroll_state,
        );
    } else {
        frame.render_widget(&explorer.widget(), sections[1]);
    }
    // TODO: set default file path to Documents or something?
    let file = explorer.current();
    app.collection_file_name = Some(file.name().to_string());
    app.collection_file = Some(file.clone());
}

/// draws the main block of the Decklist tab
fn draw_decklist_main(
    app: &mut App,
    frame: &mut Frame,
    chunk: Rect,
    main_block: Block,
    explorer: &mut FileExplorer,
) {
    // split into two sections - small one for info text and main for displaying file explorer
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(2)])
        .split(main_block.inner(chunk));
    if app.decklist.is_none() {
        app.decklist_status = "Please select a decklist.".to_string();
    } else {
        app.decklist_status = format!(
            "Decklist loaded successfully.  Using {}",
            app.decklist_file_name.as_ref().unwrap() // NOTE: should exist if you get to this branch
        );
    }
    let file_paragraph = Paragraph::new(app.decklist_status.clone()).wrap(Wrap { trim: true });
    frame.render_widget(main_block, chunk);
    frame.render_widget(file_paragraph, sections[0]);
    if app.decklist.is_some() {
        let mut lines: Vec<Line> = Vec::new();
        for card in app.decklist.as_ref().unwrap() {
            lines.push(Line::from(format!("{}", card)));
        }
        app.decklist_scroll_state = app.collection_scroll_state.content_length(lines.len());
        let decklist_paragraph = Paragraph::new(lines[app.decklist_scroll..].to_vec());
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));
        frame.render_widget(decklist_paragraph, sections[1]);
        frame.render_stateful_widget(
            scrollbar,
            sections[1].inner(Margin {
                horizontal: 1,
                vertical: 0,
            }),
            &mut app.decklist_scroll_state,
        );
    } else {
        frame.render_widget(&explorer.widget(), sections[1]);
    }
    // TODO: set default file path to Documents or something?
    let file = explorer.current();
    app.decklist_file_name = Some(file.name().to_string());
    app.decklist_file = Some(file.clone());
}

/// draws the main block of the missing cards tab
fn draw_missing_main(app: &mut App, frame: &mut Frame, chunk: Rect, main_block: Block) {
    if app.missing_cards.is_some() {
        let inner_area = main_block.inner(chunk);
        let mut lines: Vec<Line> = Vec::new();
        for card in app.missing_cards.clone().unwrap() {
            lines.push(Line::from(format!("{}", card)));
        }
        app.missing_scroll_state = app.missing_scroll_state.content_length(lines.len());
        let missing_paragraph = Paragraph::new(lines[app.missing_scroll..].to_vec());
        //.scroll((app.missing_scroll as u16, 0))
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));
        main_block.render(chunk, frame.buffer_mut());
        frame.render_widget(missing_paragraph, inner_area);
        frame.render_stateful_widget(scrollbar, inner_area, &mut app.missing_scroll_state);
    }
}
