/// System prompt for SKILLS semantic analysis
pub fn semantic_analysis_system() -> &'static str {
    r#"You are an expert AI programming assistant configuration analyzer. Your task is to analyze SKILLS (AI instruction sets) and identify semantic conflicts, contradictions, and overlaps between them.

A SKILL is a set of instructions that guides an AI programming assistant's behavior. When multiple SKILLS are loaded simultaneously, they may contain conflicting or contradictory instructions that could cause the AI to behave inconsistently.

You must respond in valid JSON format as specified in each prompt."#
}

/// Prompt for detecting semantic conflicts between skills
pub fn detect_conflicts_prompt(skills_json: &str) -> String {
    format!(
        r#"Analyze the following SKILLS and identify ALL semantic conflicts, contradictions, and overlaps between them.

Two instructions conflict when:
1. They give opposite or contradictory guidance for the same scenario
2. They define incompatible rules for the same aspect of coding
3. One instruction would prevent following another instruction
4. They have overlapping scope with different expectations

Even if instructions use different wording, identify them as conflicting if they would cause an AI to behave inconsistently.

SKILLS to analyze:
```json
{}
```

Respond in JSON format:
```json
{{
  "conflicts": [
    {{
      "instruction_a": {{"skill_name": "...", "instruction": "..."}},
      "instruction_b": {{"skill_name": "...", "instruction": "..."}},
      "conflict_type": "contradiction|incompatible|overlap|partial_conflict",
      "severity": "critical|high|medium|low|info",
      "description": "Clear explanation of why these instructions conflict",
      "scenario": "Example scenario where this conflict would cause problems"
    }}
  ],
  "compatible_pairs": [
    {{
      "instruction_a": {{"skill_name": "...", "instruction": "..."}},
      "instruction_b": {{"skill_name": "...", "instruction": "..."}},
      "note": "Why they are compatible or complementary"
    }}
  ]
}}
```

Be thorough. It's better to flag a potential conflict than to miss one. If no conflicts exist, return an empty conflicts array."#,
        skills_json
    )
}

/// System prompt for AI-driven merge
pub fn merge_system() -> &'static str {
    r#"You are an expert AI programming assistant configuration merger. Your task is to intelligently merge multiple SKILLS (AI instruction sets) into a single, coherent, conflict-free SKILL.

When merging:
1. Preserve all unique, non-conflicting instructions from all sources
2. For conflicting instructions, choose the most appropriate one based on context and best practices
3. When instructions can be combined, merge them into a more comprehensive instruction
4. Maintain clear categorization and priority ordering
5. Ensure the merged result is internally consistent - no instruction should contradict another

You must respond in valid JSON format as specified in each prompt."#
}

/// Prompt for AI-driven merge
pub fn merge_prompt(skills_json: &str, conflicts_json: &str) -> String {
    format!(
        r#"Merge the following SKILLS into a single coherent SKILL.

Source SKILLS:
```json
{}
```

Detected conflicts:
```json
{}
```

For each conflict, decide the best resolution:
- Choose the instruction that leads to better code quality, safety, and maintainability
- If both instructions have merit, combine them into a single, more nuanced instruction
- Always explain your reasoning

Respond in JSON format:
```json
{{
  "merged_skill": {{
    "name": "descriptive-name-for-merged-skill",
    "description": "Clear description of the merged skill's purpose",
    "instructions": [
      {{
        "command": "short-keyword",
        "content": "Full instruction text",
        "category": "Category name",
        "priority": 0-100,
        "rationale": "Why this instruction was included/chosen"
      }}
    ]
  }},
  "resolutions": [
    {{
      "conflict": "Brief description of the conflict",
      "resolution": "chosen|merged|skipped",
      "chosen_instruction": "The instruction that was kept or the merged result",
      "rationale": "Why this resolution was chosen"
    }}
  ]
}}
```"#,
        skills_json, conflicts_json
    )
}

/// System prompt for conflict resolution with user interaction
pub fn conflict_question_system() -> &'static str {
    r#"You are an expert AI programming assistant configuration advisor. Your task is to help users understand and resolve conflicts between SKILLS (AI instruction sets).

When a conflict cannot be automatically resolved, you should:
1. Clearly explain the conflict in plain language
2. Present the trade-offs of each option
3. Ask a focused question that helps the user make an informed decision
4. Suggest a recommended option with reasoning

Keep your explanations concise and actionable."#
}

/// Prompt for generating a user question about a conflict
pub fn conflict_question_prompt(
    conflict_description: &str,
    instruction_a: &str,
    instruction_b: &str,
    skill_a: &str,
    skill_b: &str,
) -> String {
    format!(
        r#"A conflict has been detected between two SKILLS that needs user input to resolve.

**Skill A**: "{}"
Instruction: "{}"

**Skill B**: "{}"
Instruction: "{}"

**Conflict**: {}

Please generate:
1. A clear explanation of this conflict (2-3 sentences)
2. A specific question for the user to decide
3. Two options (A and B) with brief trade-off descriptions
4. Your recommendation with reasoning

Respond in JSON format:
```json
{{
  "explanation": "Clear explanation of the conflict",
  "question": "The question to ask the user",
  "option_a": {{
    "label": "Short label for option A",
    "description": "What happens if they choose A",
    "trade_off": "Potential downside of choosing A"
  }},
  "option_b": {{
    "label": "Short label for option B",
    "description": "What happens if they choose B",
    "trade_off": "Potential downside of choosing B"
  }},
  "recommendation": "A or B",
  "reasoning": "Why this option is recommended"
}}
```"#,
        skill_a, instruction_a, skill_b, instruction_b, conflict_description
    )
}

/// System prompt for generating the final merged output
pub fn generate_output_system() -> &'static str {
    r#"You are an expert at writing clear, well-structured AI instruction documents. Your task is to take a merged set of instructions and produce a polished, well-organized SKILL document in Markdown format.

The output should be:
- Well-organized with clear sections and categories
- Written in imperative mood (e.g., "Always use spaces for indentation")
- Internally consistent with no contradictions
- Include relevant metadata and source attribution
- Professional and concise"#
}

/// Prompt for generating the final merged output
pub fn generate_output_prompt(merged_skill_json: &str) -> String {
    format!(
        r#"Generate a polished Markdown SKILL document from the following merged skill data:

```json
{}
```

The document should follow this structure:
1. YAML front matter with metadata
2. Title and description
3. Categorized instruction sections
4. Each instruction should be clear and actionable

Generate the complete Markdown document:"#,
        merged_skill_json
    )
}
