# Contributing to BatcherBird

Thank you for your interest in contributing to BatcherBird! This document provides guidelines for contributing to the project.

## Getting Started

### Prerequisites

- **macOS 10.15+** (Catalina or later)
- **Rust 1.77+** - Install via [rustup](https://rustup.rs/)
- **Node.js 20+** - Install via [Homebrew](https://brew.sh/) or [nvm](https://github.com/nvm-sh/nvm)
- **Tauri CLI** - `cargo install tauri-cli --version "^2.0"`

For testing:
- A MIDI device (synthesizer, keyboard, etc.)
- An audio interface

### Development Setup

```bash
# Clone the repository
git clone https://github.com/dbtreasure/batcherbird.git
cd batcherbird

# Install frontend dependencies
cd crates/batcherbird-gui
npm install

# Start development server
npm run dev
```

This starts both the Tauri backend and Vite frontend with hot reload.

### Project Structure

```
batcherbird/
├── crates/
│   ├── batcherbird-core/     # Rust audio processing library
│   ├── batcherbird-cli/      # Command-line interface (minimal)
│   └── batcherbird-gui/      # Tauri desktop application
│       ├── src/              # React TypeScript frontend
│       └── src-tauri/        # Tauri Rust backend
├── docs/                     # Documentation and plans
└── .github/workflows/        # CI/CD configuration
```

## How to Contribute

### Reporting Bugs

Before opening a bug report:
1. Check [existing issues](https://github.com/dbtreasure/batcherbird/issues) first
2. Try the latest version

When reporting, include:
- macOS version
- Audio interface model
- MIDI device model
- Steps to reproduce
- Expected vs actual behavior
- Any error messages or console output

### Suggesting Features

Open an issue with:
- Clear description of the feature
- Use case and why it would be valuable
- Any implementation ideas (optional)

### Pull Requests

1. **Fork** the repository
2. **Create a branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Make your changes**
4. **Test** your changes:
   ```bash
   # Rust tests
   cargo test --workspace

   # TypeScript check
   cd crates/batcherbird-gui && npx tsc --noEmit

   # Build check
   npm run build
   ```
5. **Commit** with clear messages:
   ```bash
   git commit -m "feat: add new feature description"
   ```
6. **Push** and open a Pull Request

### Commit Message Format

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `refactor:` - Code change that neither fixes a bug nor adds a feature
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

Examples:
```
feat: add velocity curve presets for common synthesizers
fix: resolve MIDI note-off timing issue with DW6000
docs: update README with troubleshooting section
```

## Code Style

### Rust

- Follow standard Rust conventions
- Run `cargo clippy` before committing
- Keep audio callback code minimal (no allocations)
- Use `eprintln!` only for error logging in audio callbacks

### TypeScript

- Use TypeScript strict mode (no `any` types)
- Prefer functional components with hooks
- Keep components focused and small
- Use the existing UI components from `src/components/ui/`

## Areas We Need Help

### High Priority

- **Hardware Testing** - Test with different synthesizers and report compatibility
- **Bug Reports** - Detailed reports with reproduction steps

### Medium Priority

- **Linux Support** - ALSA/JACK audio backend implementation
- **Windows Support** - WASAPI audio backend implementation
- **Documentation** - Tutorials, examples, setup guides

### Lower Priority

- **UI/UX Improvements** - Design enhancements
- **Localization** - Translations for other languages

## Questions?

- Open a [Discussion](https://github.com/dbtreasure/batcherbird/discussions) for questions
- Open an [Issue](https://github.com/dbtreasure/batcherbird/issues) for bugs or feature requests

## License

By contributing to BatcherBird, you agree that your contributions will be licensed under the [AGPL-3.0 License](LICENSE).

---

Thank you for helping make hardware sampling accessible to everyone!
