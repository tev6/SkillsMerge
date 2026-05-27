//! Interactive conflict resolution — prompt user for each conflict.
//!
//! When `--interactive` is passed to `merge`, this module handles the
//! per-conflict prompt loop. The user sees both sides of each conflict
//! and can choose A, B, custom merged text, skip both, or mark as merged
//! (keep both).

use crate::ir::{Conflict, Resolution, ResolutionChoice};
use std::io::{self, BufRead, Write};

/// Interactive prompt for every conflict. Returns (resolved, unresolved).
pub fn resolve_conflicts_interactively(
    conflicts: Vec<Conflict>,
    interactive: bool,
) -> (Vec<Conflict>, Vec<Conflict>) {
    if !interactive || conflicts.is_empty() {
        // Non-interactive path: all unresolved
        return (Vec::new(), conflicts);
    }

    let mut resolved = Vec::new();
    let mut unresolved = Vec::new();
    let total = conflicts.len();

    for (idx, mut conflict) in conflicts.into_iter().enumerate() {
        eprintln!(
            "{} Conflict {}/{}  [{:?}]  severity: {}\n\
             {}\n\
             {}\n",
            "─".repeat(54),
            idx + 1,
            total,
            conflict.conflict_type,
            conflict.severity,
            conflict.description,
            "─".repeat(54),
        );

        eprintln!(
            "  [A] {}: \"{}\"\n  [B] {}: \"{}\"\n",
            conflict.instruction_a.skill_name,
            conflict.instruction_a.content,
            conflict.instruction_b.skill_name,
            conflict.instruction_b.content,
        );

        let choice = prompt_choice();

        match choice.trim().to_lowercase().as_str() {
            "a" => {
                conflict.suggested_resolution = Some(Resolution {
                    strategy: "interactive".to_string(),
                    selected: ResolutionChoice::UseA,
                    rationale: format!(
                        "User chose instruction from '{}'",
                        conflict.instruction_a.skill_name
                    ),
                    custom_content: None,
                });
                resolved.push(conflict);
            }
            "b" => {
                conflict.suggested_resolution = Some(Resolution {
                    strategy: "interactive".to_string(),
                    selected: ResolutionChoice::UseB,
                    rationale: format!(
                        "User chose instruction from '{}'",
                        conflict.instruction_b.skill_name
                    ),
                    custom_content: None,
                });
                resolved.push(conflict);
            }
            "c" => {
                let custom_text = prompt_custom();
                if custom_text.is_empty() {
                    eprintln!("  ⚠ Empty input — skipping conflict.\n");
                    conflict.suggested_resolution = Some(Resolution {
                        strategy: "interactive".to_string(),
                        selected: ResolutionChoice::Skip,
                        rationale: "User skipped (empty custom input)".to_string(),
                        custom_content: None,
                    });
                } else {
                    conflict.suggested_resolution = Some(Resolution {
                        strategy: "interactive-custom".to_string(),
                        selected: ResolutionChoice::Merge,
                        rationale: format!("User custom merge: \"{custom_text}\""),
                        custom_content: Some(custom_text),
                    });
                }
                resolved.push(conflict);
            }
            "s" => {
                conflict.suggested_resolution = Some(Resolution {
                    strategy: "interactive".to_string(),
                    selected: ResolutionChoice::Skip,
                    rationale: "User chose to skip this conflict".to_string(),
                    custom_content: None,
                });
                resolved.push(conflict);
            }
            "m" => {
                conflict.suggested_resolution = Some(Resolution {
                    strategy: "interactive".to_string(),
                    selected: ResolutionChoice::Merge,
                    rationale: "User chose to keep both instructions".to_string(),
                    custom_content: None,
                });
                resolved.push(conflict);
            }
            other => {
                eprintln!("  ⚠ Unknown choice '{}'. Skipping.\n", other);
                unresolved.push(conflict);
            }
        }
    }

    (resolved, unresolved)
}

fn prompt_choice() -> String {
    prompt_line("  Choose [A / B / C(ustom) / S(kip) / M(erge both)]: ")
}

fn prompt_custom() -> String {
    eprintln!("  Enter custom merged text. Press Enter on an empty line to finish:");
    read_multiline("    > ")
}

/// Read a single line from stdin after printing a prompt.
fn prompt_line(msg: &str) -> String {
    let mut stdout = io::stdout();
    let _ = write!(stdout, "{}", msg);
    let _ = stdout.flush();

    let stdin = io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_err() {
        return String::new();
    }
    line.trim().to_string()
}

/// Read multiple lines until an empty line is entered.
fn read_multiline(prompt: &str) -> String {
    let mut stdout = io::stdout();
    let _ = write!(stdout, "{}", prompt);
    let _ = stdout.flush();

    let stdin = io::stdin();
    let mut lines = Vec::new();

    loop {
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line.trim_end_matches(['\n', '\r']);
                if trimmed.is_empty() {
                    break;
                }
                lines.push(trimmed.to_string());
            }
            Err(_) => break,
        }
        let _ = write!(stdout, "{}", prompt);
        let _ = stdout.flush();
    }

    lines.join("\n")
}
