use crate::ir::{Conflict, MergeStrategy, ResolutionChoice, SkillIR};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    MainMenu,
    LoadSkills,
    ViewConflicts,
    ConflictResolution,
    MergeResult,
    Settings,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuOption {
    LoadSkills,
    ViewConflicts,
    Merge,
    Settings,
    Help,
    Quit,
}

impl MainMenuOption {
    pub const ALL: [MainMenuOption; 6] = [
        MainMenuOption::LoadSkills,
        MainMenuOption::ViewConflicts,
        MainMenuOption::Merge,
        MainMenuOption::Settings,
        MainMenuOption::Help,
        MainMenuOption::Quit,
    ];

    pub fn label(&self) -> &str {
        match self {
            MainMenuOption::LoadSkills => "Load Skills",
            MainMenuOption::ViewConflicts => "View Conflicts",
            MainMenuOption::Merge => "Merge",
            MainMenuOption::Settings => "Settings",
            MainMenuOption::Help => "Help",
            MainMenuOption::Quit => "Quit",
        }
    }

    pub fn key(&self) -> &str {
        match self {
            MainMenuOption::LoadSkills => "1",
            MainMenuOption::ViewConflicts => "2",
            MainMenuOption::Merge => "3",
            MainMenuOption::Settings => "4",
            MainMenuOption::Help => "5",
            MainMenuOption::Quit => "Q",
        }
    }
}

pub struct App {
    pub mode: AppMode,
    pub should_quit: bool,
    pub skills: Vec<SkillIR>,
    pub conflicts: Vec<Conflict>,
    pub current_conflict_idx: usize,
    pub selected_menu_idx: usize,
    pub merge_strategy: MergeStrategy,
    pub status_message: String,
}

impl App {
    pub fn new(skills: Vec<SkillIR>) -> Self {
        Self {
            mode: AppMode::MainMenu,
            should_quit: false,
            skills,
            conflicts: Vec::new(),
            current_conflict_idx: 0,
            selected_menu_idx: 0,
            merge_strategy: MergeStrategy::AutoSelect,
            status_message: String::new(),
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        match self.mode {
            AppMode::MainMenu => self.handle_main_menu_key(key),
            AppMode::ViewConflicts => self.handle_conflicts_key(key),
            AppMode::ConflictResolution => self.handle_resolution_key(key),
            AppMode::Help => self.handle_help_key(key),
            AppMode::Settings => self.handle_settings_key(key),
            AppMode::LoadSkills => self.handle_load_key(key),
            AppMode::MergeResult => self.handle_merge_result_key(key),
        }
    }

    fn handle_main_menu_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Char('1') | KeyCode::Char('l') => {
                self.mode = AppMode::LoadSkills;
                self.status_message = "Enter file path or press Esc to cancel".to_string();
            }
            KeyCode::Char('2') | KeyCode::Char('c') => {
                if !self.skills.is_empty() {
                    self.conflicts = crate::conflict::detect_conflicts(&self.skills);
                    self.current_conflict_idx = 0;
                    self.mode = AppMode::ViewConflicts;
                    self.status_message = format!("Found {} conflicts", self.conflicts.len());
                } else {
                    self.status_message = "No skills loaded. Load skills first.".to_string();
                }
            }
            KeyCode::Char('3') | KeyCode::Char('m') => {
                if !self.skills.is_empty() {
                    self.conflicts = crate::conflict::detect_conflicts(&self.skills);
                    if !self.conflicts.is_empty() {
                        self.current_conflict_idx = 0;
                        self.mode = AppMode::ConflictResolution;
                        self.status_message = "Resolve conflicts to proceed".to_string();
                    } else {
                        self.status_message = "No conflicts detected. Ready to merge.".to_string();
                    }
                } else {
                    self.status_message = "No skills loaded. Load skills first.".to_string();
                }
            }
            KeyCode::Char('4') | KeyCode::Char('s') => {
                self.mode = AppMode::Settings;
            }
            KeyCode::Char('5') | KeyCode::Char('h') => {
                self.mode = AppMode::Help;
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                return true; // quit
            }
            KeyCode::Up if self.selected_menu_idx > 0 => {
                self.selected_menu_idx -= 1;
            }
            KeyCode::Down if self.selected_menu_idx < MainMenuOption::ALL.len() - 1 => {
                self.selected_menu_idx += 1;
            }
            KeyCode::Enter => {
                let option = MainMenuOption::ALL[self.selected_menu_idx];
                match option {
                    MainMenuOption::Quit => return true,
                    MainMenuOption::LoadSkills => {
                        self.mode = AppMode::LoadSkills;
                    }
                    MainMenuOption::ViewConflicts => {
                        if !self.skills.is_empty() {
                            self.conflicts = crate::conflict::detect_conflicts(&self.skills);
                            self.mode = AppMode::ViewConflicts;
                        }
                    }
                    MainMenuOption::Merge => {
                        if !self.skills.is_empty() {
                            self.conflicts = crate::conflict::detect_conflicts(&self.skills);
                            if !self.conflicts.is_empty() {
                                self.mode = AppMode::ConflictResolution;
                            }
                        }
                    }
                    MainMenuOption::Settings => self.mode = AppMode::Settings,
                    MainMenuOption::Help => self.mode = AppMode::Help,
                }
            }
            _ => {}
        }
        false
    }

    fn handle_conflicts_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = AppMode::MainMenu;
            }
            KeyCode::Up if self.current_conflict_idx > 0 => {
                self.current_conflict_idx -= 1;
            }
            KeyCode::Down if self.current_conflict_idx < self.conflicts.len().saturating_sub(1) => {
                self.current_conflict_idx += 1;
            }
            KeyCode::Enter => {
                self.mode = AppMode::ConflictResolution;
            }
            _ => {}
        }
        false
    }

    fn handle_resolution_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;

        if self.conflicts.is_empty() {
            self.mode = AppMode::MainMenu;
            return false;
        }

        match key.code {
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.resolve_current(ResolutionChoice::UseA);
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                self.resolve_current(ResolutionChoice::UseB);
            }
            KeyCode::Char('m') | KeyCode::Char('M') => {
                self.resolve_current(ResolutionChoice::Merge);
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.resolve_current(ResolutionChoice::Skip);
            }
            KeyCode::Esc => {
                self.mode = AppMode::ViewConflicts;
            }
            _ => {}
        }
        false
    }

    fn resolve_current(&mut self, choice: ResolutionChoice) {
        if self.current_conflict_idx < self.conflicts.len() {
            let conflict = &mut self.conflicts[self.current_conflict_idx];
            conflict.suggested_resolution = Some(crate::ir::Resolution {
                strategy: "interactive".to_string(),
                selected: choice,
                rationale: format!("User selected: {:?}", choice),
                custom_content: None,
            });

            // Move to next conflict or finish
            if self.current_conflict_idx < self.conflicts.len() - 1 {
                self.current_conflict_idx += 1;
                self.status_message = format!(
                    "Conflict {} of {} resolved. Moving to next.",
                    self.current_conflict_idx,
                    self.conflicts.len()
                );
            } else {
                self.status_message = "All conflicts resolved!".to_string();
                self.mode = AppMode::MergeResult;
            }
        }
    }

    fn handle_help_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
            self.mode = AppMode::MainMenu;
        }
        false
    }

    fn handle_settings_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
            self.mode = AppMode::MainMenu;
        }
        false
    }

    fn handle_load_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
            self.mode = AppMode::MainMenu;
        }
        false
    }

    fn handle_merge_result_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::KeyCode;
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
            self.mode = AppMode::MainMenu;
        }
        false
    }

    /// Consume the app and return skills + conflicts for merge
    pub fn into_parts(self) -> (Vec<SkillIR>, Vec<Conflict>) {
        (self.skills, self.conflicts)
    }
}
