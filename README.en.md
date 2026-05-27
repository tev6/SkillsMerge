# SkillsMerge

<p align="center">
  <strong>AI-Powered SKILLS Intelligent Merger</strong>
</p>

<p align="center">
  <a href="#installation">Installation</a> · <a href="#quick-start">Quick Start</a> · <a href="#configuration">Configuration</a> · <a href="#usage">Usage</a> · <a href="README.md">中文</a>
</p>

---

## What is it?

SkillsMerge is an AI-powered priority compiler for merging SKILLS (AI instruction sets) files used by AI coding assistants.

When using AI coding tools, you often accumulate many SKILLS resources. These SKILLS may contain:
- **Instruction conflicts**: Different SKILLS give contradictory guidance for the same scenario
- **Semantic contradictions**: Instructions with different wording but opposite meanings
- **Redundancy**: Multiple instructions expressing the same thing

SkillsMerge leverages Large Language Models (LLMs) to understand the **semantics** of instructions, intelligently detect conflicts, and merge them into a single, conflict-free SKILL file.

## Features

- **AI Semantic Analysis** — Understands instruction meaning, not just keywords
- **Intelligent Merging** — AI selects or combines instructions based on context and best practices
- **Multi-format Support** — Markdown / JSON / YAML / TOML input formats
- **Interactive TUI** — Terminal UI for guided conflict resolution
- **Offline Fallback** — Automatically falls back to rule-based engine when no API key is available
- **Cross-platform** — Windows / macOS / Linux

## Installation

### Build from Source

Requires Rust 1.75+ toolchain:

```bash
git clone https://github.com/your-username/SkillsMerge.git
cd SkillsMerge
cargo build --release
```

The binary is at `target/release/skillsmerge` (or `.exe` on Windows).

### Download

Visit [Releases](https://github.com/your-username/SkillsMerge/releases) to download for your platform.

## Quick Start

### 1. Configure AI

Copy the environment template and add your API key:

```bash
cp .env.example .env
```

Edit `.env`:

```env
SKILLSMERGE_API_KEY=sk-your-key-here
SKILLSMERGE_MODEL=gpt-4o
SKILLSMERGE_BASE_URL=https://api.openai.com/v1
```

### 2. Merge SKILLS

```bash
# AI-powered merge (default)
skillsmerge merge skill-a.md skill-b.md -o merged.md

# Merge entire directory
skillsmerge merge input/ -o merged.md

# Check for conflicts only
skillsmerge check input/
```

### 3. View Results

The merged file includes:
- Metadata (sources, merge strategy, conflict count)
- Resolved conflicts with AI's decision rationale
- Instructions sorted by category and priority

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SKILLSMERGE_API_KEY` | API Key (highest priority) | - |
| `OPENAI_API_KEY` | OpenAI-compatible API Key | - |
| `SKILLSMERGE_MODEL` | Model name | `gpt-4o` |
| `SKILLSMERGE_BASE_URL` | API endpoint | `https://api.openai.com/v1` |
| `SKILLSMERGE_TEMPERATURE` | Temperature parameter | `0.3` |
| `SKILLSMERGE_MAX_TOKENS` | Max tokens | `4096` |

### Supported AI Services

| Service | BASE_URL | Notes |
|---------|----------|-------|
| OpenAI | `https://api.openai.com/v1` | Official API |
| DeepSeek | `https://api.deepseek.com/v1` | Cost-effective |
| Azure OpenAI | `https://<resource>.openai.azure.com/...` | Enterprise |
| Ollama Local | `http://localhost:11434/v1` | Fully offline |

**Priority**: CLI args > Environment variables > Config file > Defaults

## Usage

### Command Line

```bash
# AI merge (default strategy)
skillsmerge merge skill-a.md skill-b.md -o merged.md

# Specify model
skillsmerge merge input/ -o merged.md --ai-model deepseek-chat

# Specify API endpoint
skillsmerge merge input/ -o merged.md --ai-base-url https://api.deepseek.com/v1

# Use rule-based engine (no AI)
skillsmerge merge input/ -o merged.md --strategy auto

# Conflict detection
skillsmerge check input/

# Interactive TUI
skillsmerge interactive input/

# Batch processing
skillsmerge batch --config rules.toml input/ -o output/
```

### Merge Strategies

| Strategy | Description |
|----------|-------------|
| `ai` | AI-driven merge (default), understands semantics, makes intelligent decisions |
| `auto` | Rule-based, auto-selects by priority |
| `preserve-all` | Keeps all instructions, only marks conflicts |
| `interactive` | Resolve conflicts one by one |
| `semantic` | Semantic merge, attempts to combine instructions |

### Input Formats

Four SKILLS file formats are supported:

**Markdown** (`.md`):
```markdown
# Skill Name
Description: Skill description

## Instructions
- Instruction 1
- Instruction 2

## Configuration
```json
{ "priority": 10 }
```
```

**JSON** (`.json`), **YAML** (`.yaml`/`.yml`), and **TOML** (`.toml`) are also supported.

## How It Works

```
Input SKILLS files
       │
       ▼
  Parse to Intermediate Representation (SkillIR)
       │
       ▼
  AI Semantic Conflict Detection ──→ LLM understands meaning, finds deep conflicts
       │
       ▼
  AI Intelligent Merge ──→ LLM decides best approach based on context
       │
       ▼
  AI Output Generation ──→ LLM produces polished merged document
```

When no API key is available, it automatically falls back to rule-based mode (command name matching and priority comparison).

## Development

```bash
# Development build
cargo build

# Run tests
cargo test

# Run example
cargo run --example basic_merge

# Lint
cargo clippy --all-targets --all-features
```

## License

[MIT](LICENSE)
