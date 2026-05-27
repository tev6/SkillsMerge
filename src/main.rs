use std::process;

use clap::Parser;
use skillsmerge::ai::client::LlmClient;
use skillsmerge::cli::{self, Commands};
use skillsmerge::config;
use skillsmerge::error::SkillsMergeError;
use skillsmerge::io;
use skillsmerge::ir::MergeStrategy;
use skillsmerge::merger;
use skillsmerge::output;
use skillsmerge::reporter;

#[tokio::main]
async fn main() {
    // Load .env file if present
    let _ = dotenvy::dotenv();

    let args = cli::parse_args();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .init();

    if let Err(e) = run(args).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

async fn run(args: cli::Cli) -> Result<(), SkillsMergeError> {
    match args.command {
        Some(Commands::Merge {
            input,
            output,
            strategy,
            format,
            auto_resolve,
            ai_model,
            ai_base_url,
            api_key,
            ai_output,
        }) => {
            run_merge(
                &input,
                output.as_deref(),
                &strategy,
                &format,
                auto_resolve.as_deref(),
                ai_model.as_deref(),
                ai_base_url.as_deref(),
                api_key.as_deref(),
                ai_output,
            )
            .await
        }
        Some(Commands::Check {
            input,
            format,
            ai_model,
            ai_base_url,
            api_key,
        }) => {
            run_check(
                &input,
                &format,
                ai_model.as_deref(),
                ai_base_url.as_deref(),
                api_key.as_deref(),
            )
            .await
        }
        Some(Commands::Batch {
            config: config_path,
            input,
            output,
        }) => run_batch(&config_path, &input, output.as_deref()).await,
        Some(Commands::Interactive {
            input,
            ai_model,
            api_key,
        }) => run_interactive(&input, ai_model.as_deref(), api_key.as_deref()),
        None => {
            cli::Cli::try_parse_from(["skillsmerge", "--help"]).ok();
            Ok(())
        }
    }
}

fn build_llm_config(
    ai_model: Option<&str>,
    ai_base_url: Option<&str>,
    api_key: Option<&str>,
) -> skillsmerge::config::AiConfig {
    let mut ai_config = skillsmerge::config::AiConfig::default();
    if let Some(model) = ai_model {
        ai_config.model = model.to_string();
    }
    if let Some(base_url) = ai_base_url {
        ai_config.base_url = base_url.to_string();
    }
    if let Some(key) = api_key {
        ai_config.api_key = Some(key.to_string());
    }
    ai_config
}

#[allow(clippy::too_many_arguments)]
async fn run_merge(
    input: &[std::path::PathBuf],
    output: Option<&std::path::Path>,
    strategy_str: &str,
    format_str: &str,
    auto_resolve: Option<&str>,
    ai_model: Option<&str>,
    ai_base_url: Option<&str>,
    api_key: Option<&str>,
    ai_output: bool,
) -> Result<(), SkillsMergeError> {
    // Load skills
    let (skills, errors) = io::load_skills(input);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("Warning: {}", e);
        }
    }

    if skills.is_empty() {
        return Err(SkillsMergeError::FileNotFound {
            path: "No valid SKILLS files found".to_string(),
        });
    }

    println!("Loaded {} skill(s)", skills.len());

    let result = if strategy_str == "ai" {
        // AI-driven merge
        let ai_config = build_llm_config(ai_model, ai_base_url, api_key);
        let llm_config = ai_config.to_llm_config();
        println!(
            "Using AI model: {} ({})",
            llm_config.model, llm_config.base_url
        );
        let client = LlmClient::new(llm_config);

        let result = skillsmerge::ai::merge::ai_merge(&client, skills).await?;

        // Optionally use AI to generate polished output
        if ai_output {
            println!("AI is generating polished output...");
            let ai_content = skillsmerge::ai::merge::generate_ai_output(&client, &result).await?;

            match output {
                Some(path) => {
                    io::ensure_output_dir(path)?;
                    output::write_to_file(&ai_content, path)?;
                    println!("AI-generated output written to: {}", path.display());
                }
                None => {
                    println!("{}", ai_content);
                }
            }

            // Print summary
            let summary = reporter::generate_merge_summary(&result);
            eprintln!("\n{}", summary);
            return Ok(());
        }

        result
    } else {
        // Rule-based merge (fallback)
        let strategy = if let Some(ar) = auto_resolve {
            MergeStrategy::Custom(ar.to_string())
        } else {
            strategy_str.parse().unwrap_or(MergeStrategy::AutoSelect)
        };
        merger::merge(skills, &strategy)
    };

    // Output
    let output_format = format_str
        .parse()
        .unwrap_or(skillsmerge::ir::OutputFormat::Markdown);
    let content = output::generate(&result, output_format);

    match output {
        Some(path) => {
            io::ensure_output_dir(path)?;
            output::write_to_file(&content, path)?;
            println!("Output written to: {}", path.display());
        }
        None => {
            println!("{}", content);
        }
    }

    // Print summary
    let summary = reporter::generate_merge_summary(&result);
    eprintln!("\n{}", summary);

    if !result.conflicts_unresolved.is_empty() {
        return Err(SkillsMergeError::ConflictUnresolved {
            count: result.conflicts_unresolved.len(),
        });
    }

    Ok(())
}

async fn run_check(
    input: &[std::path::PathBuf],
    format_str: &str,
    ai_model: Option<&str>,
    ai_base_url: Option<&str>,
    api_key: Option<&str>,
) -> Result<(), SkillsMergeError> {
    let (skills, errors) = io::load_skills(input);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("Warning: {}", e);
        }
    }

    if skills.is_empty() {
        return Err(SkillsMergeError::FileNotFound {
            path: "No valid SKILLS files found".to_string(),
        });
    }

    println!("Checking {} skill(s) for conflicts...", skills.len());

    // Try AI-based conflict detection
    let ai_config = build_llm_config(ai_model, ai_base_url, api_key);
    let llm_config = ai_config.to_llm_config();

    let conflicts = if !llm_config.api_key.is_empty() {
        println!("Using AI semantic analysis...");
        let client = LlmClient::new(llm_config);
        skillsmerge::ai::semantic::detect_semantic_conflicts(&client, &skills).await?
    } else {
        println!("No API key found, using rule-based conflict detection...");
        skillsmerge::conflict::detect_conflicts(&skills)
    };

    if conflicts.is_empty() {
        println!("No conflicts detected.");
        return Ok(());
    }

    println!("Found {} conflict(s):\n", conflicts.len());

    let report = reporter::generate_conflict_report(&conflicts);
    let output_format = format_str
        .parse()
        .unwrap_or(skillsmerge::ir::OutputFormat::Markdown);

    match output_format {
        skillsmerge::ir::OutputFormat::Markdown => println!("{}", report),
        skillsmerge::ir::OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&conflicts).unwrap_or_default();
            println!("{}", json);
        }
        _ => println!("{}", report),
    }

    Ok(())
}

async fn run_batch(
    config_path: &std::path::Path,
    input: &[std::path::PathBuf],
    output: Option<&std::path::Path>,
) -> Result<(), SkillsMergeError> {
    let cfg = config::load_config(config_path)?;

    let output_path = output
        .map(|p| p.to_path_buf())
        .or_else(|| Some(std::path::PathBuf::from("output")));

    run_merge(
        input,
        output_path.as_deref(),
        &cfg.default_strategy,
        &cfg.default_output_format,
        cfg.auto_resolve.as_deref(),
        Some(&cfg.ai.model),
        Some(&cfg.ai.base_url),
        cfg.ai.api_key.as_deref(),
        true,
    )
    .await
}

fn run_interactive(
    input: &[std::path::PathBuf],
    ai_model: Option<&str>,
    api_key: Option<&str>,
) -> Result<(), SkillsMergeError> {
    let (skills, errors) = io::load_skills(input);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("Warning: {}", e);
        }
    }

    let ai_config = build_llm_config(ai_model, None, api_key);
    if ai_config.api_key.is_some()
        || std::env::var("SKILLSMERGE_API_KEY").is_ok()
        || std::env::var("OPENAI_API_KEY").is_ok()
    {
        println!("AI assistance enabled (model: {})", ai_config.model);
    }

    println!(
        "Starting interactive mode with {} skill(s)...",
        skills.len()
    );
    skillsmerge::tui::run(skills)
}
