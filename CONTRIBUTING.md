# Contributing to Agent Paradise Standards System

Thank you for your interest in contributing! This guide will help you get started.

## Quick Start

```bash
# 1. Fork and clone
git clone https://github.com/YOUR_USERNAME/agent-paradise-standards-system.git
cd agent-paradise-standards-system

# 2. Set up development environment
just init

# 3. Make your changes...

# 4. Run checks before committing
just check

# 5. Commit with conventional message
git commit -m "feat: add my awesome feature"

# 6. Push and open PR
git push origin my-feature-branch
```

## Development Setup

### Prerequisites

- **Rust 1.85+** — Install via [rustup](https://rustup.rs/)
- **Just** — Task runner: `cargo install just`

### Useful Commands

| Command | Description |
|---------|-------------|
| `just check` | Format, lint, and test |
| `just build` | Build all crates |
| `just aps-validate` | Validate all standards |
| `just aps-list` | List discovered packages |

## Creating a New Standard

All new standards start as **experiments** to allow iteration before promotion.

### 1. Create an Experiment

```bash
aps v1 create experiment my-idea
```

This scaffolds:
```
standards-experimental/v1/EXP-V1-XXXX-my-idea/
├── experiment.toml
├── docs/
│   ├── 00_overview.md
│   └── 01_spec.md
├── examples/
├── tests/
├── agents/skills/
├── src/lib.rs
└── Cargo.toml
```

### 2. Implement Required Structure

Every standard must include:

- **`docs/01_spec.md`** — Core specification
- **`examples/`** — At least one working example
- **`tests/`** — Automated validation tests
- **`agents/skills/`** — Agent capability definitions

### 3. Validate Your Work

```bash
aps v1 validate repo
```

This checks:
- Required files exist
- Metadata is valid
- Naming conventions followed

### 4. Open a PR

- Use a descriptive title
- Reference any related issues
- Ensure `just check` passes

### 5. Promotion to Official Standard

After community review and iteration, experiments can be promoted:

```bash
aps v1 promote EXP-V1-XXXX
```

This moves the experiment to `standards/v1/APS-V1-XXXX-slug/`.

## Code Contributions

### Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

| Prefix | Use For |
|--------|---------|
| `feat:` | New features |
| `fix:` | Bug fixes |
| `docs:` | Documentation changes |
| `refactor:` | Code restructuring |
| `test:` | Test additions/changes |
| `chore:` | Maintenance tasks |

Examples:
```
feat(cli): add version bump command
fix(core): handle missing manifest gracefully
docs: update contributing guide
```

### Pull Request Checklist

Before submitting:

- [ ] `just check` passes (format, lint, test)
- [ ] `aps v1 validate repo` passes (if touching standards)
- [ ] Commit messages follow conventions
- [ ] Added/updated relevant documentation
- [ ] Added tests for new functionality

## For AI Agents

See [AGENTS.md](AGENTS.md) for the RIPER-5 operational protocol designed for AI coding assistants.

Key points:
- Always declare your current mode
- Follow the Research → Innovate → Plan → Execute → Review flow
- Use `just check` after Execute mode
- Commit with conventional messages

## Getting Help

- **Issues** — Bug reports and feature requests
- **Discussions** — Questions and ideas
- **PRs** — Code contributions

## License

By contributing, you agree that your contributions will be licensed under the [Apache-2.0 License](LICENSE).

