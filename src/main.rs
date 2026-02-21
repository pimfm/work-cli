mod agents;
mod app;
mod cli;
mod config;
mod event;
mod model;
mod providers;
mod ui;
mod util;

use std::io;
use std::panic;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use app::{Action, App};

#[tokio::main]
async fn main() -> Result<()> {
    // Check for CLI subcommands before launching TUI
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "add" => return cli::handle_add(&args[2..]).await,
            "help" | "--help" | "-h" => {
                cli::print_help();
                return Ok(());
            }
            _ => {} // Unknown subcommand — fall through to TUI
        }
    }

    // Load config
    let config = config::load_config()?;

    // Initialize agent store
    let store = agents::store::AgentStore::new()?;

    // Set up action channel
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();

    // Create app
    let mut app = App::new(&config, store, action_tx.clone());

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // Set up panic hook to restore terminal
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Spawn event reader
    let event_tx = action_tx.clone();
    tokio::spawn(async move {
        event::run_event_loop(event_tx).await;
    });

    // Initial fetch: if no board mapping, show picker; otherwise load items
    if app.view_mode == app::ViewMode::BoardSelection {
        app.fetch_boards().await;
    } else {
        app.refresh_items().await;
    }

    // Main loop
    loop {
        // Show/hide cursor based on input mode
        if app.input_active {
            terminal.show_cursor()?;
        } else {
            terminal.hide_cursor()?;
        }

        // Render
        terminal.draw(|f| ui::render(f, &app))?;

        // Wait for action
        if let Some(action) = action_rx.recv().await {
            app.update(action).await;
            if app.should_quit {
                break;
            }
        } else {
            break;
        }
    }

    // Restore terminal
    terminal.show_cursor()?;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
