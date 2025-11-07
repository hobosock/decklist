use std::io;

pub mod app;
pub mod collection;
pub mod config;
pub mod database;
pub mod startup;
pub mod tui;

use app::App;
use ratatui_explorer::{FileExplorer, Theme};
use tui::core::{init, restore};

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let mut terminal = init()?;
    let theme = Theme::default().add_default_title();
    let mut collection_explorer = FileExplorer::with_theme(theme.clone())?;
    let mut deck_explorer = FileExplorer::with_theme(theme.clone())?;
    let mut database_explorer = FileExplorer::with_theme(theme)?;
    let app_result = App::default().run(
        &mut terminal,
        &mut collection_explorer,
        &mut deck_explorer,
        &mut database_explorer,
    );
    restore()?;
    app_result
}
