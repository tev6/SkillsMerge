use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::{App, AppMode, MainMenuOption};
use crate::ir::MergeStrategy;

pub fn draw(f: &mut Frame, app: &mut App) {
    match app.mode {
        AppMode::MainMenu => draw_main_menu(f, app),
        AppMode::ViewConflicts => draw_conflicts(f, app),
        AppMode::ConflictResolution => draw_conflict_resolution(f, app),
        AppMode::Help => draw_help(f, app),
        AppMode::Settings => draw_settings(f, app),
        AppMode::LoadSkills => draw_load_skills(f, app),
        AppMode::MergeResult => draw_merge_result(f, app),
    }
}

fn draw_main_menu(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("SkillsMerge v1.0.0 - Interactive Merge")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Menu items
    let items: Vec<ListItem> = MainMenuOption::ALL
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let style = if i == app.selected_menu_idx {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(
                format!("  [{}] {}", opt.key(), opt.label()),
                style,
            )))
        })
        .collect();

    let menu = List::new(items).block(Block::default().borders(Borders::ALL).title("Menu"));
    f.render_widget(menu, chunks[1]);

    // Status bar
    let status = Paragraph::new(Line::from(vec![
        Span::styled("Skills: ", Style::default().fg(Color::Gray)),
        Span::styled(
            app.skills.len().to_string(),
            Style::default().fg(Color::White),
        ),
        Span::raw("  "),
        Span::styled("Conflicts: ", Style::default().fg(Color::Gray)),
        Span::styled(
            app.conflicts.len().to_string(),
            Style::default().fg(Color::White),
        ),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}

fn draw_conflicts(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new(format!("Conflicts ({})", app.conflicts.len()))
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Conflict list
    let items: Vec<ListItem> = app
        .conflicts
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let style = if i == app.current_conflict_idx {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(Span::styled(
                format!("[{}] {} - {}", c.severity, c.conflict_type, c.description),
                style,
            )))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Conflict List"),
    );
    f.render_widget(list, chunks[1]);

    // Help bar
    let help = Paragraph::new("↑/↓: Navigate  Enter: Resolve  Esc: Back")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[2]);
}

fn draw_conflict_resolution(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Title
    let conflict = app.conflicts.get(app.current_conflict_idx);
    let title_text = conflict
        .map(|c| {
            format!(
                "Conflict #{}: {}",
                app.current_conflict_idx + 1,
                c.conflict_type
            )
        })
        .unwrap_or_else(|| "No conflicts".to_string());

    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Source A
    let source_a = conflict
        .map(|c| {
            format!(
                "Skill: {}\nCommand: {}\nContent: {}",
                c.instruction_a.skill_name, c.instruction_a.command, c.instruction_a.content
            )
        })
        .unwrap_or_default();
    let para_a = Paragraph::new(source_a)
        .block(Block::default().borders(Borders::ALL).title("Source A"))
        .wrap(Wrap { trim: true });
    f.render_widget(para_a, chunks[1]);

    // Source B
    let source_b = conflict
        .map(|c| {
            format!(
                "Skill: {}\nCommand: {}\nContent: {}",
                c.instruction_b.skill_name, c.instruction_b.command, c.instruction_b.content
            )
        })
        .unwrap_or_default();
    let para_b = Paragraph::new(source_b)
        .block(Block::default().borders(Borders::ALL).title("Source B"))
        .wrap(Wrap { trim: true });
    f.render_widget(para_b, chunks[2]);

    // Options
    let options = Paragraph::new(
        "[A] Use Source A    [B] Use Source B    [M] Merge    [S] Skip    [Esc] Back",
    )
    .style(Style::default().fg(Color::Cyan))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(options, chunks[3]);
}

fn draw_help(f: &mut Frame, _app: &App) {
    let help_text = vec![
        "SkillsMerge v1.0.0 - Help",
        "",
        "Keyboard Shortcuts:",
        "  1/L - Load Skills files",
        "  2/C - View detected conflicts",
        "  3/M - Start merge process",
        "  4/S - Settings",
        "  5/H - This help screen",
        "  Q   - Quit application",
        "",
        "Conflict Resolution:",
        "  A - Use Source A's instruction",
        "  B - Use Source B's instruction",
        "  M - Merge both instructions",
        "  S - Skip this conflict",
        "",
        "Navigation:",
        "  ↑/↓ - Move up/down in lists",
        "  Enter - Select/Confirm",
        "  Esc - Go back",
        "",
        "Press Esc to return to main menu",
    ];

    let paragraph = Paragraph::new(help_text.join("\n"))
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, f.area());
}

fn draw_settings(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(f.area());

    let title = Paragraph::new("Settings")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let strategies = [
        "Preserve All  — keep every instruction, flag all conflicts",
        "Auto Select   — auto-resolve via priority comparison",
        "Interactive   — prompt user for each conflict",
        "Semantic Merge — use heuristics to merge related instructions",
    ];
    let selected_idx = match app.merge_strategy {
        MergeStrategy::PreserveAll => 0usize,
        MergeStrategy::AutoSelect => 1,
        MergeStrategy::Interactive => 2,
        MergeStrategy::SemanticMerge => 3,
        MergeStrategy::Custom(_) => 0,
    };
    let items: Vec<ListItem> = strategies
        .iter()
        .enumerate()
        .map(|(i, desc)| {
            let prefix = if i == selected_idx { "▶ " } else { "  " };
            let style = if i == selected_idx {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", prefix, desc),
                style,
            )))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Merge Strategy (← → to change)"),
    );
    f.render_widget(list, chunks[1]);

    let help = if !app.status_message.is_empty() {
        app.status_message.clone()
    } else {
        "← → : switch strategy    Esc : back to menu".to_string()
    };
    let help_p = Paragraph::new(help)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help_p, chunks[2]);
}

fn draw_load_skills(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    let title = Paragraph::new(format!(
        "Load Skills — {} skill(s) currently loaded",
        app.skills.len()
    ))
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Show the input prompt and current buffer
    let prompt = if app.input_buffer.is_empty() {
        "Type a file or directory path and press Enter to load.\n\
         Example: tests/fixtures/skills\n\
         Example: skill-a.md"
            .to_string()
    } else {
        format!("> {}", app.input_buffer)
    };
    let input = Paragraph::new(prompt)
        .block(Block::default().borders(Borders::ALL).title("File Path"))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(input, chunks[1]);

    // Status / loaded skill list
    let status_text = if !app.status_message.is_empty() {
        app.status_message.clone()
    } else {
        let names: Vec<String> = app.skills.iter().map(|s| s.name.clone()).collect();
        if names.is_empty() {
            "No skills loaded yet.".to_string()
        } else {
            format!("Loaded: {}", names.join(", "))
        }
    };
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}

fn draw_merge_result(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(f.area());

    let title = Paragraph::new("Merge Result")
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let resolved = app
        .conflicts
        .iter()
        .filter(|c| c.suggested_resolution.is_some())
        .count();
    let total = app.conflicts.len();

    let body = if app.save_mode {
        let prompt = if app.input_buffer.is_empty() {
            "Enter filename to save the merged output:".to_string()
        } else {
            format!("> {}", app.input_buffer)
        };
        format!(
            "Conflicts resolved: {}/{}\n\
             Skills: {}\n\n\
             Saving merged output...\n\n\
             {}",
            resolved,
            total,
            app.skills.len(),
            prompt,
        )
    } else {
        format!(
            "Conflicts resolved: {}/{}\n\
             Skills: {}\n\
             Instructions: {} (after merge)\n\n\
             All conflicts have been resolved.\n\n\
             [S] Save to file    [Esc] Back to menu",
            resolved,
            total,
            app.skills.len(),
            app.skills
                .iter()
                .map(|s| s.instructions.len())
                .sum::<usize>(),
        )
    };

    let paragraph = Paragraph::new(body)
        .block(Block::default().borders(Borders::ALL).title("Summary"))
        .style(if app.save_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        })
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[1]);

    let help = if !app.status_message.is_empty() {
        app.status_message.clone()
    } else if app.save_mode {
        "Type filename and press Enter, or Esc to cancel".to_string()
    } else {
        "[S] Save to file    [Esc] Back to menu".to_string()
    };
    let help_p = Paragraph::new(help)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help_p, chunks[2]);
}
