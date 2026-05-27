```yaml
---
name: Unified-Code-Style
description: >-
  A comprehensive set of coding style guidelines that resolve conflicts between
  indentation, naming conventions, and line length preferences by adopting
  widely accepted standards and providing context-sensitive rules.
resolutions:
  - conflict: |
      Skill A mandates tabs for indentation, while Skill B mandates spaces.
      These are mutually exclusive.

      Scenario: When the AI needs to format code, it cannot use both tabs and
      spaces for indentation simultaneously. Choosing one would violate the other.
    resolution: Merge
    rationale: AI determined this resolution
  - conflict: |
      Skill A requires snake_case naming convention, while Skill B requires
      camelCase. They cannot both be applied to the same variable names.

      Scenario: When creating a variable, the AI must decide between snake_case
      and camelCase. Following both leads to inconsistent naming.
    resolution: Merge
    rationale: AI determined this resolution
  - conflict: |
      Skill A sets a maximum line length of 120 characters, while Skill B sets
      a stricter maximum of 80 characters. These constraints are incompatible
      because a line longer than 80 characters but shorter than 120 would satisfy
      A but violate B.

      Scenario: The AI writes a line of code that is 100 characters long. It
      complies with Skill A's limit but violates Skill B's limit, causing
      uncertainty on which rule to prioritize.
    resolution: Merge
    rationale: AI determined this resolution
source: merged-skills
---
# Unified Code Style

A comprehensive set of coding style guidelines that resolve conflicts between indentation, naming conventions, and line length preferences by adopting widely accepted standards and providing context-sensitive rules.

## Formatting

### Indentation
Always use spaces for indentation as the default (4 spaces per level). If the language’s official style guide (e.g., Python PEP 8) or an existing project convention mandates tabs, use tabs consistently and configure your editor to display a tab width of 4 spaces. Never mix tabs and spaces in the same file.

### Line Length
The hard maximum line length is 120 characters. Strive to keep lines under 80 characters where possible, especially for comments, documentation, and function signatures, as shorter lines improve readability and side‑by‑side diff comparisons. When breaking lines, maintain syntactic and semantic clarity.

## Naming

### Variable Naming
Follow the standard naming convention for the programming language in use:

- Use `camelCase` for languages like Java, JavaScript, C#, and TypeScript.
- Use `snake_case` for Python, Ruby, and similar languages.

If a project‑specific style guide specifies a different convention, adhere to that guide. Maintain consistency within the entire codebase.