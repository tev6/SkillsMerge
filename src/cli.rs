use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "skillsmerge")]
#[command(version = "1.0.0")]
#[command(
    about = "An AI-powered SKILLS merger - uses LLM to intelligently merge and resolve conflicts"
)]
#[command(
    after_help = "Examples:\n  skillsmerge merge skill-a.md skill-b.md -o merged.md\n  skillsmerge merge input/ -o merged.md --ai-model gpt-4o\n  skillsmerge check input/*.md\n  skillsmerge batch --config rules.toml input/ -o output/\n\nAPI Key:\n  Set SKILLSMERGE_API_KEY or OPENAI_API_KEY environment variable,\n  or use --api-key flag, or configure in config file."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Merge multiple SKILLS files into one using AI
    Merge {
        /// Input SKILLS files or directories
        #[arg(required = true, num_args = 1..)]
        input: Vec<std::path::PathBuf>,

        /// Output file path
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Merge strategy: ai, auto, preserve-all, interactive, semantic
        #[arg(short, long, default_value = "ai")]
        strategy: String,

        /// Output format: markdown, json, yaml, toml
        #[arg(short = 'f', long, default_value = "markdown")]
        format: String,

        /// Auto-resolve strategy for batch mode: latest, highest-priority
        #[arg(long)]
        auto_resolve: Option<String>,

        /// AI model to use (e.g., gpt-4o, gpt-4o-mini)
        #[arg(long)]
        ai_model: Option<String>,

        /// AI API base URL (for OpenAI-compatible APIs)
        #[arg(long)]
        ai_base_url: Option<String>,

        /// AI API key (overrides environment variable)
        #[arg(long)]
        api_key: Option<String>,

        /// Use AI to generate polished output
        #[arg(long, default_value = "false")]
        ai_output: bool,
    },

    /// Check for conflicts using AI semantic analysis
    Check {
        /// Input SKILLS files or directories
        #[arg(required = true, num_args = 1..)]
        input: Vec<std::path::PathBuf>,

        /// Output format for conflict report
        #[arg(short = 'f', long, default_value = "markdown")]
        format: String,

        /// AI model to use
        #[arg(long)]
        ai_model: Option<String>,

        /// AI API base URL
        #[arg(long)]
        ai_base_url: Option<String>,

        /// AI API key
        #[arg(long)]
        api_key: Option<String>,
    },

    /// Batch process using a configuration file
    Batch {
        /// Configuration file path
        #[arg(short, long)]
        config: std::path::PathBuf,

        /// Input SKILLS files or directories
        #[arg(required = true, num_args = 1..)]
        input: Vec<std::path::PathBuf>,

        /// Output directory
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },

    /// Launch interactive TUI mode with AI assistance
    Interactive {
        /// Input SKILLS files or directories to pre-load
        #[arg(num_args = 0..)]
        input: Vec<std::path::PathBuf>,

        /// Output file path (saves merged result after exit)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// AI model to use
        #[arg(long)]
        ai_model: Option<String>,

        /// AI API key
        #[arg(long)]
        api_key: Option<String>,
    },
}

/// Parse CLI arguments
pub fn parse_args() -> Cli {
    Cli::parse()
}
