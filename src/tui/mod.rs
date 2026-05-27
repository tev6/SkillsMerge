pub mod app;
pub mod ui;

use crate::error::Result;
use crate::ir::SkillIR;

/// Run the TUI application
pub fn run(skills: Vec<SkillIR>) -> Result<()> {
    let mut app = app::App::new(skills);

    crossterm::terminal::enable_raw_mode().map_err(|e| {
        crate::error::SkillsMergeError::IoError(std::io::Error::other(e.to_string()))
    })?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)
        .map_err(crate::error::SkillsMergeError::IoError)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend).map_err(|e| {
        crate::error::SkillsMergeError::IoError(std::io::Error::other(e.to_string()))
    })?;

    // Main loop
    loop {
        terminal.draw(|f| ui::draw(f, &mut app)).map_err(|e| {
            crate::error::SkillsMergeError::IoError(std::io::Error::other(e.to_string()))
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))
            .map_err(crate::error::SkillsMergeError::IoError)?
        {
            if let crossterm::event::Event::Key(key) =
                crossterm::event::read().map_err(crate::error::SkillsMergeError::IoError)?
            {
                if app.handle_key(key) {
                    break;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    crossterm::terminal::disable_raw_mode().map_err(|e| {
        crate::error::SkillsMergeError::IoError(std::io::Error::other(e.to_string()))
    })?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )
    .map_err(crate::error::SkillsMergeError::IoError)?;

    Ok(())
}
