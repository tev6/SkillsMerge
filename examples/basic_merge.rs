use skillsmerge::io;
use skillsmerge::ir::MergeStrategy;
use skillsmerge::merger;
use skillsmerge::output;

fn main() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/skills");

    // Load all skills from fixtures
    let (skills, errors) = io::load_skills(&[dir]);

    if !errors.is_empty() {
        eprintln!("Errors loading skills:");
        for e in &errors {
            eprintln!("  - {}", e);
        }
    }

    println!("Loaded {} skill(s):", skills.len());
    for skill in &skills {
        println!(
            "  - {} ({} instructions, format: {})",
            skill.name,
            skill.instructions.len(),
            skill.source.format
        );
    }

    // Merge with auto strategy
    let result = merger::merge(skills, &MergeStrategy::AutoSelect);

    println!("\nMerge result:");
    println!(
        "  Total instructions: {}",
        result.statistics.total_instructions
    );
    println!("  Conflicts found: {}", result.statistics.conflicts_found);
    println!("  Conflicts resolved: {}", result.conflicts_resolved.len());
    println!(
        "  Conflicts unresolved: {}",
        result.conflicts_unresolved.len()
    );
    println!("  Duration: {}ms", result.statistics.merge_duration_ms);

    // Generate output
    let content = output::generate(&result, skillsmerge::ir::OutputFormat::Markdown);
    println!("\n--- Merged Output ---\n");
    println!("{}", content);
}
