mod app;
mod db;
mod export;
mod types;
mod ui;
mod worker;

use anyhow::{Context, Result};
use app::App;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use db::Database;
use export::{export, ExportFormat};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

#[derive(Parser)]
#[command(name = "sqr")]
#[command(about = "A fast, keyboard-first TUI for exploring SQLite databases")]
struct Cli {
    /// Database file path
    #[arg(value_name = "DATABASE")]
    database: Option<String>,

    /// Open database in read-write mode
    #[arg(long)]
    read_write: bool,

    /// Number of rows per page
    #[arg(long, default_value = "100")]
    page_size: usize,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Export data from database
    Export {
        /// Database file path
        #[arg(long, short)]
        db: String,

        /// Table name to export
        #[arg(long, short)]
        table: Option<String>,

        /// SQL query to execute
        #[arg(long, short)]
        query: Option<String>,

        /// Output format
        #[arg(long, short, value_enum)]
        format: ExportFormatArg,

        /// Output file path
        #[arg(long, short)]
        out: String,
    },
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum ExportFormatArg {
    Csv,
    Json,
}

impl From<ExportFormatArg> for ExportFormat {
    fn from(fmt: ExportFormatArg) -> Self {
        match fmt {
            ExportFormatArg::Csv => ExportFormat::Csv,
            ExportFormatArg::Json => ExportFormat::Json,
        }
    }
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    // Handle export command
    if let Some(Commands::Export {
        db,
        table,
        query,
        format,
        out,
    }) = cli.command
    {
        return run_export(&db, table.as_deref(), query.as_deref(), format.into(), &out);
    }

    // Handle TUI mode
    let db_path = cli.database.context("Database path is required")?;
    run_tui(&db_path, cli.read_write, cli.page_size)
}

fn run_export(
    db_path: &str,
    table: Option<&str>,
    query: Option<&str>,
    format: ExportFormat,
    output_path: &str,
) -> Result<()> {
    let database = Database::new(db_path, false)?;
    let conn = database.into_connection();

    export(
        &conn,
        format,
        std::path::Path::new(output_path),
        table,
        query,
    )?;

    println!("Exported to: {}", output_path);
    Ok(())
}

fn run_tui(db_path: &str, read_write: bool, page_size: usize) -> Result<()> {
    // Open database
    // Database::new expects read_only flag, so we pass !read_write
    // If read_write is true, we want read_only=false (read-write mode)
    // If read_write is false, we want read_only=true (read-only mode)
    let database = Database::new(db_path, !read_write)
        .with_context(|| format!("Failed to open database: {}", db_path))?;

    // Create worker with database connection
    let worker = worker::Worker::new(database.into_connection());

    // Create app
    let mut app = App::new(worker, page_size);

    // Load initial tables
    app.load_tables();

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Main event loop
    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if app.should_quit() {
            break;
        }

        // Process worker responses
        app.process_worker_responses()?;

        // Handle input and resize events
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    app.handle_key_event(key)?;
                }
                Event::Resize(_, _) => {
                    // Terminal will automatically redraw on next draw() call
                }
                _ => {}
            }
        }
    }

    // Cleanup
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    app.shutdown()?;

    Ok(())
}

