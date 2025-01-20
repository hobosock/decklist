use std::io;

pub mod app;
pub mod config;
pub mod startup;
pub mod tui;

use ratatui_explorer::{FileExplorer, Theme};
use tui::core::{init, restore};

use app::App;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let mut terminal = init()?;
    let theme = Theme::default().add_default_title();
    let mut collection_explorer = FileExplorer::with_theme(theme.clone())?;
    let mut deck_explorer = FileExplorer::with_theme(theme)?;
    let app_result =
        App::default().run(&mut terminal, &mut collection_explorer, &mut deck_explorer);
    restore()?;
    app_result
}
