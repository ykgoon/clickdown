//! Terminal initialization and management

use std::io;
use crossterm::{
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
    ExecutableCommand,
};
use ratatui::{Terminal, backend::CrosstermBackend, Frame};
use anyhow::Result;

/// Initialize the terminal for TUI rendering
pub fn init() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    // Enter alternate screen
    io::stdout().execute(EnterAlternateScreen)?;
    
    // Enter raw mode for direct input capture
    terminal::enable_raw_mode()?;
    
    // Create backend and terminal
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    
    // Clear the screen
    io::stdout().execute(Clear(ClearType::All))?;
    
    Ok(terminal)
}

/// Restore terminal to original state
pub fn restore() -> Result<()> {
    // Leave alternate screen
    io::stdout().execute(LeaveAlternateScreen)?;
    
    // Disable raw mode
    terminal::disable_raw_mode()?;
    
    // Clear the screen
    io::stdout().execute(Clear(ClearType::All))?;
    
    Ok(())
}

/// Draw a frame on the terminal
pub fn draw<F>(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, f: F) -> Result<()>
where
    F: FnOnce(&mut Frame),
{
    terminal.draw(f)?;
    Ok(())
}
