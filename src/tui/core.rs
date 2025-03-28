use std::io::{self, stdout, Stdout};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    prelude::{CrosstermBackend, Widget},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, Tabs, Wrap},
    Frame, Terminal,
};
use ratatui_explorer::FileExplorer;

use crate::{app::App, database::scryfall::PriceType};

use super::{
    help::{ABOUT_STR, BUG_STR, HELP_STR},
    space_padding,
};

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
    database_explorer: &mut FileExplorer,
) {
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
    let version =
        Line::from(vec!["| Decklist v0.3.0 |".into()]).style(Style::default().cyan().bold());
    let main_block = Block::default()
        .title_bottom(version)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border::THICK);

    // change bottom two chunks based on selected tab
    let instructions_block = Block::default().borders(Borders::ALL);
    let mut instructions_text = Text::from(vec![Line::from(vec![
        "<Q>".yellow().bold(),
        " Quit ".into(),
        "<1-6>".yellow().bold(),
        " Change Tabs ".into(),
    ])]);
    match app.active_tab {
        MenuTabs::Welcome => {
            draw_welcome_main(app, frame, chunks[1], main_block);
        }
        MenuTabs::Database => {
            instructions_text = Text::from(vec![Line::from(vec![
                "<Q>".yellow().bold(),
                " Quit ".into(),
                "<S>".yellow().bold(),
                " Load file ".into(),
                "<Up/Down>".yellow().bold(),
                " Navigate ".into(),
                "<Left/Backspace>".yellow().bold(),
                " Exit Directory ".into(),
                "<Right/Enter>".yellow().bold(),
                " Down Directory ".into(),
            ])]);
            draw_database_main(app, frame, chunks[1], main_block, database_explorer);
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
                "<F>".yellow().bold(),
                " Save to file ".into(),
                "<Up/Down>".yellow().bold(),
                " Navigate ".into(),
            ])]);
            draw_missing_main(app, frame, chunks[1], main_block);
        }
        MenuTabs::Help => {
            draw_help_main(frame, chunks[1], main_block);
        }
        MenuTabs::Debug => {
            draw_debug_main(app, frame, chunks[1], main_block);
        }
    }

    let instructions = Paragraph::new(instructions_text)
        .centered()
        .block(instructions_block);

    // render
    frame.render_widget(tabs, chunks[0]);
    frame.render_widget(instructions, chunks[2]);
}

fn draw_debug_main(app: &mut App, frame: &mut Frame, chunk: Rect, main_block: Block) {
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(main_block.inner(chunk));
    let debug_text = Paragraph::new(Text::from(app.debug_string.clone())).wrap(Wrap { trim: true });
    let counter_paragraph = Paragraph::new(vec![
        Line::from(vec![
            Span::from("Directory Check: ").bold(),
            Span::from(space_padding(6)),
            Span::from(format!("{}", app.directory_counter)).cyan(),
        ]),
        Line::from(vec![
            Span::from("Config Check: ").bold(),
            Span::from(space_padding(9)),
            Span::from(format!("{}", app.config_counter)).cyan(),
        ]),
        Line::from(vec![
            Span::from("Database Check: ").bold(),
            Span::from(space_padding(7)),
            Span::from(format!("{}", app.database_counter)).cyan(),
        ]),
        Line::from(vec![
            Span::from("Collection Check: ").bold(),
            Span::from(space_padding(5)),
            Span::from(format!("{}", app.collection_counter)).cyan(),
        ]),
        Line::from(vec![
            Span::from("Decklist Check: ").bold(),
            Span::from(space_padding(7)),
            Span::from(format!("{}", app.decklist_counter)).cyan(),
        ]),
        Line::from(vec![
            Span::from("Legal Check: ").bold(),
            Span::from(space_padding(10)),
            Span::from(format!("{}", app.legal_counter)).cyan(),
        ]),
        Line::from(vec![
            Span::from("Missing Check: ").bold(),
            Span::from(space_padding(8)),
            Span::from(format!("{}", app.missing_counter)).cyan(),
        ]),
        Line::from(vec![
            Span::from("Price Check: ").bold(),
            Span::from(space_padding(10)),
            Span::from(format!("{}", app.price_counter)).cyan(),
        ]),
    ]);
    frame.render_widget(main_block, chunk);
    frame.render_widget(debug_text, sections[0]);
    frame.render_widget(counter_paragraph, sections[1]);
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
    let database_status_line = if app.database_ok {
        app.dc.database_status.clone().green()
    } else {
        app.dc.database_status.clone().red()
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
    //.centered()
    .block(main_block);
    frame.render_widget(status_paragraph, chunk);
}

/// draws the main block of the Database tab
fn draw_database_main(
    app: &mut App,
    frame: &mut Frame,
    chunk: Rect,
    main_block: Block,
    explorer: &mut FileExplorer,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(2)])
        .split(main_block.inner(chunk));
    let file_paragraph = app.dc.database_status.clone();
    frame.render_widget(main_block, chunk);
    frame.render_widget(file_paragraph, sections[0]);
    frame.render_widget(&explorer.widget(), sections[1]);
    let file = explorer.current();
    app.man_database_file_name = Some(file.name().to_string());
    app.man_database_file = Some(file.clone());
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
    if app.collection.is_some() && app.collection_file_name.is_some() {
        app.collection_status = format!(
            "Collection loaded successfully.  Using {}",
            app.collection_file_name.as_ref().unwrap() // NOTE: should exist if you get to this branch
        );
    }
    let file_paragraph = if app.prompt_config_update {
        let words = app.collection_status.clone()
            + "\nPress C to update config to auto load this collection file.";
        Paragraph::new(words).wrap(Wrap { trim: true })
    } else {
        Paragraph::new(app.collection_status.clone()).wrap(Wrap { trim: true })
    };
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
    let decklist_msg = if app.decklist.is_none() {
        format!("Please select a decklist. | {}", app.decklist_status)
    } else {
        format!(
            "Decklist loaded successfully.  Using {}",
            app.decklist_file_name.as_ref().unwrap() // NOTE: should exist if you get to this branch
        )
    };
    let file_paragraph = Paragraph::new(decklist_msg).wrap(Wrap { trim: true });
    frame.render_widget(main_block, chunk);
    frame.render_widget(file_paragraph, sections[0]);
    if app.decklist.is_some() {
        // further split area  for format legality info
        let subs = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Min(3)])
            .split(sections[1]);
        let mut lines: Vec<Line> = Vec::new();
        for card in app.decklist.as_ref().unwrap() {
            lines.push(Line::from(format!("{}", card)));
        }
        app.decklist_scroll_state = app.collection_scroll_state.content_length(lines.len());
        let decklist_paragraph = Paragraph::new(lines[app.decklist_scroll..].to_vec());
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));
        frame.render_widget(decklist_paragraph, subs[0]);
        frame.render_stateful_widget(
            scrollbar,
            subs[0].inner(Margin {
                horizontal: 1,
                vertical: 0,
            }),
            &mut app.decklist_scroll_state,
        );
        // style text based on legality
        if app.legality.is_some() {
            let fl = app.legality.as_ref().unwrap();
            // this is annoyingly redundant but spans have lifetimes, so making this a function is
            // even more annoying ¯\_(ツ)_/¯
            let standard_text = if fl.standard {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let future_text = if fl.future {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let historic_text = if fl.historic {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let timeless_text = if fl.timeless {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let gladiator_text = if fl.gladiator {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let pioneer_text = if fl.pioneer {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let explorer_text = if fl.explorer {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let modern_text = if fl.modern {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let legacy_text = if fl.legacy {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let pauper_text = if fl.pauper {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let vintage_text = if fl.vintage {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let penny_text = if fl.penny {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let commander_text = if fl.commander {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let oathbreaker_text = if fl.oathbreaker {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let standard_brawl_text = if fl.standardbrawl {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let brawl_text = if fl.brawl {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let alchemy_text = if fl.alchemy {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let pauper_commander_text = if fl.paupercommander {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let duel_text = if fl.duel {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let oldschool_text = if fl.oldschool {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let premodern_text = if fl.premodern {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let predh_text = if fl.predh {
                Span::from("LEGAL").green()
            } else {
                Span::from("NOT LEGAL").red()
            };
            let legal_lines = Paragraph::new(vec![
                Line::from(vec![Span::from("Standard: ").bold(), standard_text]),
                Line::from(vec![Span::from("Pioneer: ").bold(), pioneer_text]),
                Line::from(vec![Span::from("Modern: ").bold(), modern_text]),
                Line::from(vec![Span::from("Legacy: ").bold(), legacy_text]),
                Line::from(vec![Span::from("Vintage: ").bold(), vintage_text]),
                Line::from(vec![Span::from("Pauper: ").bold(), pauper_text]),
                Line::from(vec![Span::from("Penny: ").bold(), penny_text]),
                Line::from(vec![Span::from("Premodern: ").bold(), premodern_text]),
                Line::from(vec![Span::from("Old School: ").bold(), oldschool_text]),
                Line::from(vec![Span::from("Commander: ").bold(), commander_text]),
                Line::from(vec![
                    Span::from("Pauper Commander: ").bold(),
                    pauper_commander_text,
                ]),
                Line::from(vec![Span::from("Explorer: ").bold(), explorer_text]),
                Line::from(vec![Span::from("Historic: ").bold(), historic_text]),
                Line::from(vec![Span::from("Timeless: ").bold(), timeless_text]),
                Line::from(vec![Span::from("Alchemy: ").bold(), alchemy_text]),
                Line::from(vec![Span::from("Brawl: ").bold(), brawl_text]),
                Line::from(vec![
                    Span::from("Standard Brawl: ").bold(),
                    standard_brawl_text,
                ]),
                Line::from(vec![Span::from("Predh: ").bold(), predh_text]),
                Line::from(vec![Span::from("Gladiator: ").bold(), gladiator_text]),
                Line::from(vec![Span::from("Duel: ").bold(), duel_text]),
                Line::from(vec![Span::from("Future: ").bold(), future_text]),
                Line::from(vec![Span::from("Oathbreaker: ").bold(), oathbreaker_text]),
            ]);
            frame.render_widget(legal_lines, subs[1]);
        }
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
        /* moving for speed
        let mut lines: Vec<Line> = Vec::new();
        for card in app.missing_cards.clone().unwrap() {
            let missing_str = if app.card_database.is_some() {
                check_missing(&app.card_database.as_ref().unwrap(), &card)
            } else {
                "".to_string()
            };
            lines.push(Line::from(format!("{}{}", card, missing_str)));
        }
        */
        app.missing_scroll_state = app
            .missing_scroll_state
            .content_length(app.missing_lines.len());
        let mut missing_lines = Vec::new();
        // find longest missing line for padding
        let mut spacing: usize = 0;
        for card in app.missing_lines.iter() {
            let length = card.len(); // NOTE: get character length? card.chars().count()
            if length > spacing {
                spacing = length;
            }
        }
        spacing += 5;
        for (i, line_str) in app.missing_lines.iter().enumerate() {
            let price_str = if app.price_done
                && app.missing_price.is_some()
                && app.missing_price.as_ref().unwrap().len() > i
            {
                app.missing_price.as_ref().unwrap()[i].clone()
            } else {
                "".to_string()
            };
            missing_lines.push(Line::from(vec![
                Span::from(line_str.clone()),
                Span::from(space_padding(spacing - line_str.len())),
                Span::from(price_str).magenta(),
            ]));
        }
        // add final total
        missing_lines.push(Line::from("\n"));
        let currency_str = match app.config.currency {
            PriceType::USD => "$".to_string(),
            PriceType::Euro => "€".to_string(),
            PriceType::Tix => "Tix ".to_string(),
        };
        missing_lines.push(Line::from(vec![
            Span::from("Total: ").light_red().bold().underlined(),
            Span::from(space_padding(spacing - 7)),
            Span::from(currency_str).light_red().bold().underlined(),
            Span::from(format!(
                "{:.2}",
                app.missing_price_num
                    .as_ref()
                    .unwrap_or(&vec![0.0]) // TODO: this unwrap is lazy
                    .iter()
                    .sum::<f64>()
            ))
            .light_red()
            .bold()
            .underlined(),
        ]));
        let missing_paragraph = Paragraph::new(missing_lines[app.missing_scroll..].to_vec());
        //.scroll((app.missing_scroll as u16, 0))
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"));
        main_block.render(chunk, frame.buffer_mut());
        frame.render_widget(missing_paragraph, inner_area);
        frame.render_stateful_widget(scrollbar, inner_area, &mut app.missing_scroll_state);
    } else {
        if app.decklist.is_some() && !app.waiting_for_missing {
            let missing_paragraph = Paragraph::new("No missing cards!").block(main_block);
            frame.render_widget(missing_paragraph, chunk);
        } else if app.decklist.is_some() {
            let missing_paragraph =
                Paragraph::new("Checking for missing cards...").block(main_block);
            frame.render_widget(missing_paragraph, chunk);
        } else {
            let missing_paragraph = Paragraph::new("Load a decklist first.").block(main_block);
            frame.render_widget(missing_paragraph, chunk);
        }
    }
}

/// draws the main block of the help tab
fn draw_help_main(frame: &mut Frame, chunk: Rect, main_block: Block) {
    let subs = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(2), Constraint::Min(2), Constraint::Min(2)])
        .split(main_block.inner(chunk));
    let help_paragraph = Paragraph::new(HELP_STR).wrap(Wrap { trim: true });
    let about_paragraph = Paragraph::new(ABOUT_STR).wrap(Wrap { trim: true }).cyan();
    let bug_paragraph = Paragraph::new(BUG_STR).wrap(Wrap { trim: true }).magenta();
    frame.render_widget(help_paragraph, subs[0]);
    frame.render_widget(about_paragraph, subs[1]);
    frame.render_widget(bug_paragraph, subs[2]);
}
