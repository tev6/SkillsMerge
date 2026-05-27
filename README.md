# SkillsMerge

<p align="center">
  <strong>AI 驱动的 SKILLS 智能合并器</strong>
</p>

<p align="center">
  <a href="#安装">安装</a> · <a href="#快速开始">快速开始</a> · <a href="#配置">配置</a> · <a href="#使用方法">使用方法</a> · <a href="README.en.md">English</a>
</p>

---

## 它是什么？

SkillsMerge 是一个 AI 驱动的优先级编译器，用于合并多个 AI 编程工具的 SKILLS（技能指令集）文件。

当你在使用 AI 编程工具时，往往会积累大量 SKILLS 资源。这些 SKILLS 之间可能存在：
- **指令冲突**：不同 SKILLS 对同一场景给出矛盾指令
- **语义矛盾**：措辞不同但含义相反的指令
- **重复冗余**：多条指令表达相同含义

SkillsMerge 利用大语言模型（LLM）理解指令的**语义**，智能检测冲突并合并，生成无矛盾的统一 SKILL 文件。

## 特性

- **AI 语义分析** — 不仅匹配关键词，更理解指令含义，发现深层矛盾
- **智能合并** — AI 根据上下文和最佳实践自动选择或组合指令
- **多格式支持** — Markdown / JSON / YAML / TOML 四种输入格式
- **交互式 TUI** — 终端界面引导冲突解决
- **离线回退** — 无 API Key 时自动回退到规则引擎模式
- **跨平台** — Windows / macOS / Linux

## 安装

### 从源码构建

需要 Rust 1.75+ 工具链：

```bash
git clone https://github.com/tev6/SkillsMerge.git
cd SkillsMerge
cargo build --release
```

构建产物位于 `target/release/skillsmerge`（或 `.exe`）。

## 快速开始

### 1. 配置 AI

复制环境变量模板并填入你的 API Key：

```bash
cp .env.example .env
```

编辑 `.env`：

```env
SKILLSMERGE_API_KEY=sk-your-key-here
SKILLSMERGE_MODEL=gpt-4o
SKILLSMERGE_BASE_URL=https://api.openai.com/v1
```

### 2. 合并 SKILLS

```bash
# AI 智能合并（默认）
skillsmerge merge skill-a.md skill-b.md -o merged.md

# 合并整个目录
skillsmerge merge input/ -o merged.md

# 仅检测冲突
skillsmerge check input/
```

### 3. 查看结果

合并后的文件包含：
- 元数据（来源、合并策略、冲突数量）
- 已解决的冲突及 AI 的决策理由
- 按类别和优先级排序的指令

## 配置

### 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `SKILLSMERGE_API_KEY` | API Key（优先级最高） | - |
| `OPENAI_API_KEY` | OpenAI 兼容 API Key | - |
| `SKILLSMERGE_MODEL` | 模型名称 | `gpt-4o` |
| `SKILLSMERGE_BASE_URL` | API 地址 | `https://api.openai.com/v1` |
| `SKILLSMERGE_TEMPERATURE` | 温度参数 | `0.3` |
| `SKILLSMERGE_MAX_TOKENS` | 最大 token 数 | `4096` |

### 支持的 AI 服务

| 服务 | BASE_URL | 说明 |
|------|----------|------|
| OpenAI | `https://api.openai.com/v1` | 官方 API |
| DeepSeek | `https://api.deepseek.com/v1` | 性价比高 |
| Azure OpenAI | `https://<resource>.openai.azure.com/...` | 企业级 |
| Ollama 本地 | `http://localhost:11434/v1` | 完全离线 |

**优先级**：CLI 参数 > 环境变量 > 配置文件 > 默认值

## 使用方法

### 命令行

```bash
# AI 合并（默认策略）
skillsmerge merge skill-a.md skill-b.md -o merged.md

# 指定模型
skillsmerge merge input/ -o merged.md --ai-model deepseek-chat

# 指定 API 地址
skillsmerge merge input/ -o merged.md --ai-base-url https://api.deepseek.com/v1

# 使用规则引擎（不调用 AI）
skillsmerge merge input/ -o merged.md --strategy auto

# 冲突检测
skillsmerge check input/

# 交互式 TUI
skillsmerge interactive input/ -o merged.md

# 批量处理
skillsmerge batch --config rules.toml input/ -o output/
```

### 交互式 TUI 工作流

交互模式让你在终端中手动审查并解决冲突：

```bash
skillsmerge interactive skill-a.md skill-b.md -o merged.md
```

**TUI 键盘快捷键：**

| 按键 | 功能 |
|------|------|
| `1`/`L` | 加载技能文件 |
| `2`/`C` | 查看检测到的冲突 |
| `3`/`M` | 开始合并流程 |
| `4`/`S` | 设置 |
| `5`/`H` | 帮助 |
| `Q` | 退出 |

**冲突解决时的快捷键：**

| 按键 | 功能 |
|------|------|
| `A` | 使用来源 A 的指令 |
| `B` | 使用来源 B 的指令 |
| `M` | 合并两条指令 |
| `S` | 跳过此冲突 |
| `Esc` | 返回 |

解决所有冲突后，合并结果会自动保存到 `-o` 指定的文件，或输出到终端。

### 输出示例

合并后的输出包含结构化的文档，附有每个冲突解决的解释：

```markdown
# Merged Skill Collection

## Metadata
- Original Skills: Merged skill from: Skill A, Skill B, skill-c
- Total Instructions: 7
- Merge Duration: 0ms
- Resolution: all conflicts resolved

## Conflicts

### Conflict #1: instruction_override
> **Source**: Skill A vs Skill B
> **Resolution**: UseB (auto-select)
> **Rationale**: Selected by skill priority (10 vs 20)

## Instructions

### Category: Security

- **Never** (priority: 20): Never commit secrets to version control
- **Always** (priority: 18): Always validate user input

### Category: Style

- **Use** (priority: 5): Use descriptive variable names
```

### 合并策略

| 策略 | 说明 |
|------|------|
| `ai` | AI 驱动合并（默认），理解语义，智能决策 |
| `auto` | 规则引擎，按优先级自动选择 |
| `preserve-all` | 保留所有指令，仅标记冲突 |
| `interactive` | 交互式逐个解决冲突 |
| `semantic` | 语义合并，尝试组合指令 |

### 输入格式

支持四种 SKILLS 文件格式：

**Markdown**（`.md`）：
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

**JSON**（`.json`）、**YAML**（`.yaml`/`.yml`）、**TOML**（`.toml`）同样支持。

## 工作原理

```
输入 SKILLS 文件
       │
       ▼
  解析为中间表示（SkillIR）
       │
       ▼
  AI 语义冲突检测 ──→ LLM 理解指令含义，发现深层矛盾
       │
       ▼
  AI 智能合并 ──→ LLM 根据上下文决定最佳方案
       │
       ▼
  AI 生成输出 ──→ LLM 生成精炼的合并文档
```

无 API Key 时自动回退到规则引擎模式（基于命令名匹配和优先级比较）。

## 开发

```bash
# 开发构建
cargo build

# 运行测试
cargo test

# 运行示例
cargo run --example basic_merge

# 代码检查
cargo clippy --all-targets --all-features
```

## 许可证

[MIT](LICENSE)
