# BatcherBird v0.1.0 Release Preparation - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prepare BatcherBird for public GitHub release with clean code, security audit, working CI/CD, and honest documentation.

**Architecture:** Phase-based approach - Discovery (parallel sub-agents), Remediation (fix findings), Infrastructure (CI/CD), Documentation, Release.

**Tech Stack:** Rust (workspace with 3 crates), React 19 + TypeScript, Tauri 2.6, GitHub Actions

---

## Phase 1: Discovery (Parallel Sub-Agents)

These tasks run in parallel using sub-agents. Each produces a findings report.

---

### Task 1: Code Review - Rust Crates

**Agent Type:** Explore (thorough)

**Scope:** Review all Rust code for release readiness

**Files to examine:**
- `crates/batcherbird-core/src/*.rs` (24 files)
- `crates/batcherbird-gui/src-tauri/src/*.rs` (3 files)
- `crates/batcherbird-cli/src/main.rs`

**What to find:**

1. **Dead code**: Unused functions, modules, or files
2. **Debug artifacts**: `println!`, `dbg!`, `eprintln!` that shouldn't be in release
3. **TODO/FIXME comments**: List all with file:line
4. **Hardcoded paths**: Any `/Users/david/` or similar
5. **Unwrap/expect abuse**: Panics that should be proper error handling
6. **Commented-out code blocks**: More than 3 lines
7. **Unused dependencies**: In Cargo.toml files

**Output:** Create `docs/reports/code-review-rust.md` with findings organized by severity (P0/P1/P2/P3)

---

### Task 2: Code Review - React Frontend

**Agent Type:** Explore (thorough)

**Scope:** Review all TypeScript/React code for release readiness

**Files to examine:**
- `crates/batcherbird-gui/src/*.tsx` (2 files)
- `crates/batcherbird-gui/src/components/*.tsx` (16 files)
- `crates/batcherbird-gui/src/hooks/*.ts` (2 files)
- `crates/batcherbird-gui/src/types/*.ts` (1 file)

**What to find:**

1. **console.log/debug statements**: Should not be in production
2. **TODO/FIXME comments**: List all with file:line
3. **Hardcoded values**: URLs, paths, test data
4. **Unused imports/components**: Dead code
5. **TypeScript `any` usage**: Should be properly typed
6. **Error handling gaps**: Missing try/catch, unhandled promise rejections
7. **Accessibility issues**: Missing aria labels, keyboard navigation

**Output:** Create `docs/reports/code-review-frontend.md` with findings organized by severity

---

### Task 3: Security Audit

**Agent Type:** general-purpose

**Scope:** Comprehensive security review

**Step 1: Check for secrets in git history**

Run:
```bash
cd /Users/david/Github/batcherbird
git log -p --all -S 'password' --source --all | head -100
git log -p --all -S 'secret' --source --all | head -100
git log -p --all -S 'api_key' --source --all | head -100
git log -p --all -S 'token' --source --all | head -100
git grep -n 'Bearer ' || echo "No bearer tokens found"
```

**Step 2: Check for environment files**

Run:
```bash
find /Users/david/Github/batcherbird -name ".env*" -o -name "*.env" 2>/dev/null
find /Users/david/Github/batcherbird -name "credentials*" -o -name "*secret*" 2>/dev/null
```

**Step 3: Dependency audit**

Run:
```bash
cd /Users/david/Github/batcherbird && cargo audit 2>&1 || echo "Install with: cargo install cargo-audit"
cd /Users/david/Github/batcherbird/crates/batcherbird-gui && npm audit 2>&1
```

**Step 4: Review Tauri security config**

Examine `crates/batcherbird-gui/src-tauri/tauri.conf.json`:
- Check CSP policy
- Review asset protocol scope
- Check for overly permissive capabilities

**Step 5: Review Tauri commands**

Examine `crates/batcherbird-gui/src-tauri/src/lib.rs`:
- List all `#[tauri::command]` functions
- Check for path traversal vulnerabilities in file operations
- Check input validation

**Step 6: Check .gitignore**

Verify sensitive patterns are ignored:
```bash
cat /Users/david/Github/batcherbird/.gitignore
```

**Output:** Create `docs/reports/security-audit.md` with:
- Secrets found (CRITICAL)
- Dependency vulnerabilities (HIGH/MEDIUM/LOW)
- Tauri security assessment
- Recommendations

---

### Task 4: Static Analysis - Rust

**Agent Type:** Bash

**Scope:** Run Rust linting and analysis tools

**Step 1: Run clippy**

```bash
cd /Users/david/Github/batcherbird && cargo clippy --workspace --all-targets 2>&1 | tee /tmp/clippy-output.txt
```

**Step 2: Check for warnings**

```bash
cd /Users/david/Github/batcherbird && cargo check --workspace 2>&1 | grep -E "^warning" | head -50
```

**Step 3: Run tests**

```bash
cd /Users/david/Github/batcherbird && cargo test --workspace 2>&1 | tee /tmp/test-output.txt
```

**Output:** Save results to `docs/reports/static-analysis-rust.md`

---

### Task 5: Static Analysis - Frontend

**Agent Type:** Bash

**Scope:** Run TypeScript/ESLint analysis

**Step 1: TypeScript strict check**

```bash
cd /Users/david/Github/batcherbird/crates/batcherbird-gui && npx tsc --noEmit 2>&1 | head -100
```

**Step 2: Check for lint config**

```bash
ls -la /Users/david/Github/batcherbird/crates/batcherbird-gui/.eslint* /Users/david/Github/batcherbird/crates/batcherbird-gui/eslint* 2>/dev/null || echo "No ESLint config found"
```

**Step 3: Build check**

```bash
cd /Users/david/Github/batcherbird/crates/batcherbird-gui && npm run build 2>&1
```

**Output:** Save results to `docs/reports/static-analysis-frontend.md`

---

## Phase 2: Remediation

After Phase 1 completes, review all findings reports and fix issues by priority.

---

### Task 6: Fix P0 Issues (Blockers)

**Files:** Determined by Phase 1 findings

**Step 1: Review all P0 findings**

Read:
- `docs/reports/code-review-rust.md`
- `docs/reports/code-review-frontend.md`
- `docs/reports/security-audit.md`

**Step 2: Fix each P0 issue**

For each issue:
1. Read the affected file
2. Make the fix
3. Verify the fix (run relevant check)
4. Commit with message: `fix: [description] (P0)`

**Step 3: Verify no regressions**

Run:
```bash
cd /Users/david/Github/batcherbird && cargo check --workspace
cd /Users/david/Github/batcherbird/crates/batcherbird-gui && npm run build
```

---

### Task 7: Fix P1 Issues (Embarrassing)

**Files:** Determined by Phase 1 findings

Same process as Task 6, but for P1 issues.

Commit with message: `fix: [description] (P1)`

---

### Task 8: Convert Remaining Issues to GitHub Issues

**Step 1: For each P2/P3 finding, create a markdown list**

Format:
```markdown
## Deferred Issues for v0.1.0

### P2 - Annoying
- [ ] [file:line] Description of issue

### P3 - Nice to Have
- [ ] [file:line] Description of issue
```

**Step 2: Add to docs/KNOWN_ISSUES.md**

This documents what we're aware of but deferring.

---

## Phase 3: Infrastructure

---

### Task 9: Create GitHub Actions CI Workflow

**File:** Create `.github/workflows/ci.yml`

**Step 1: Create the workflow file**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  rust:
    name: Rust Checks
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: clippy

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Check
        run: cargo check --workspace

      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

      - name: Test
        run: cargo test --workspace

  frontend:
    name: Frontend Checks
    runs-on: macos-latest
    defaults:
      run:
        working-directory: crates/batcherbird-gui
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: crates/batcherbird-gui/package-lock.json

      - name: Install dependencies
        run: npm ci

      - name: Type check
        run: npx tsc --noEmit

      - name: Build
        run: npm run build

  build:
    name: Build App
    runs-on: macos-latest
    needs: [rust, frontend]
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install frontend dependencies
        working-directory: crates/batcherbird-gui
        run: npm ci

      - name: Build Tauri app
        working-directory: crates/batcherbird-gui
        run: npm run tauri build

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: batcherbird-macos
          path: crates/batcherbird-gui/src-tauri/target/release/bundle/dmg/*.dmg
```

**Step 2: Test locally first**

```bash
cd /Users/david/Github/batcherbird && cargo check --workspace && cargo clippy --workspace
cd /Users/david/Github/batcherbird/crates/batcherbird-gui && npm run build
```

**Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add GitHub Actions workflow for CI"
```

---

### Task 10: Create GitHub Actions Release Workflow

**File:** Create `.github/workflows/release.yml`

**Step 1: Create the workflow file**

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  build-macos:
    name: Build macOS
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install frontend dependencies
        working-directory: crates/batcherbird-gui
        run: npm ci

      - name: Build Tauri app
        working-directory: crates/batcherbird-gui
        run: npm run tauri build

      - name: Rename DMG
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          mv crates/batcherbird-gui/src-tauri/target/release/bundle/dmg/*.dmg \
             BatcherBird-${VERSION}-macos.dmg

      - name: Upload release asset
        uses: softprops/action-gh-release@v1
        with:
          files: BatcherBird-*.dmg
          draft: true
          generate_release_notes: true
```

**Step 2: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add GitHub Actions release workflow"
```

---

### Task 11: Verify Build Works Locally

**Step 1: Full build test**

```bash
cd /Users/david/Github/batcherbird/crates/batcherbird-gui
npm ci
npm run tauri build 2>&1 | tee /tmp/build-output.txt
```

**Step 2: Check artifact exists**

```bash
ls -la /Users/david/Github/batcherbird/crates/batcherbird-gui/src-tauri/target/release/bundle/dmg/
```

**Step 3: Document any build issues**

If build fails, document in `docs/reports/build-issues.md`

---

## Phase 4: Documentation

---

### Task 12: Update License References

**Files to update:**
- `Cargo.toml` (workspace)
- `crates/batcherbird-core/Cargo.toml`
- `crates/batcherbird-cli/Cargo.toml`
- `crates/batcherbird-gui/Cargo.toml`
- `crates/batcherbird-gui/src-tauri/Cargo.toml`
- `crates/batcherbird-gui/package.json`
- `README.md`

**Step 1: Update workspace Cargo.toml**

Add after `[workspace]`:
```toml
[workspace.package]
license = "AGPL-3.0-or-later"
repository = "https://github.com/YOUR_USERNAME/batcherbird"
```

**Step 2: Update each crate's Cargo.toml**

Add to `[package]`:
```toml
license.workspace = true
repository.workspace = true
```

**Step 3: Update package.json**

Change `"license": "ISC"` to `"license": "AGPL-3.0-or-later"`

**Step 4: Update README.md**

Change license section to reference AGPL-3.0

**Step 5: Commit**

```bash
git add -A
git commit -m "docs: update license to AGPL-3.0 across all files"
```

---

### Task 13: Update README.md

**File:** `README.md`

**Step 1: Fix placeholder URLs**

Replace all `yourusername/batcherbird` with actual GitHub username/repo

**Step 2: Add Known Limitations section**

Add before "Contributing":

```markdown
## ⚠️ Current Limitations

BatcherBird is in early release (v0.1.0). Please be aware:

- **macOS only** - Windows and Linux support planned for future releases
- **Unsigned app** - You'll need to right-click → Open on first launch (Gatekeeper)
- **Hardware testing** - While core features work, extended sampling sessions haven't been battle-tested
- **No auto-update** - Check GitHub releases for new versions

We'd love your feedback! Please [open an issue](https://github.com/YOUR_USERNAME/batcherbird/issues) if you encounter problems.
```

**Step 3: Add Troubleshooting section**

Add after Known Limitations:

```markdown
## 🔧 Troubleshooting

### "App is damaged" or Gatekeeper warning
This is normal for unsigned apps. Right-click the app and select "Open", then click "Open" in the dialog.

### No MIDI devices detected
- Ensure your MIDI interface is connected before launching
- Try unplugging and reconnecting the MIDI interface
- Check System Preferences → Security & Privacy → Privacy → Input Monitoring

### No audio input detected
- Grant microphone permission when prompted
- Check System Preferences → Security & Privacy → Privacy → Microphone
- Verify your audio interface is selected as input in System Preferences → Sound
```

**Step 4: Commit**

```bash
git add README.md
git commit -m "docs: update README with limitations and troubleshooting"
```

---

### Task 14: Create CHANGELOG.md

**File:** Create `CHANGELOG.md`

```markdown
# Changelog

All notable changes to BatcherBird will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-XX

### Added
- **Professional Audio Engine**
  - 32-bit float WAV export
  - Sub-millisecond MIDI timing precision
  - Zero-dropout recording with persistent streams
  - Lock-free real-time audio architecture

- **Recording Modes**
  - Single note recording with custom velocity/duration
  - Range recording (batch sample entire octaves)
  - Velocity layer sampling (2/3/4 layers + custom)

- **Real-Time Visualization**
  - 60fps waveform display during recording
  - Professional VU meters with peak/RMS
  - Color-coded level zones with clipping detection

- **Sample Processing**
  - Automatic sample detection and trimming
  - Intelligent loop detection (FFT-based)
  - Multi-algorithm detection engine

- **Export Formats**
  - DecentSampler (.dspreset)
  - SFZ 2.0
  - Professional WAV with metadata

- **User Interface**
  - Professional dark theme
  - Device auto-detection
  - Keyboard shortcuts (spacebar play/pause)
  - Toast notifications

### Known Issues
- macOS only (Windows/Linux planned)
- App is unsigned (Gatekeeper warning on first launch)
- Extended sampling sessions not yet battle-tested

[0.1.0]: https://github.com/YOUR_USERNAME/batcherbird/releases/tag/v0.1.0
```

**Commit:**

```bash
git add CHANGELOG.md
git commit -m "docs: add CHANGELOG for v0.1.0"
```

---

### Task 15: Create CONTRIBUTING.md

**File:** Create `CONTRIBUTING.md`

```markdown
# Contributing to BatcherBird

Thank you for your interest in contributing to BatcherBird!

## Getting Started

### Prerequisites
- macOS 10.15+ (Catalina or later)
- Rust 1.70+
- Node.js 20+
- A MIDI device and audio interface (for testing)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/batcherbird.git
cd batcherbird

# Install frontend dependencies
cd crates/batcherbird-gui
npm install

# Start development
npm run dev
```

## How to Contribute

### Reporting Bugs
- Check existing issues first
- Include your macOS version, audio interface, and MIDI device
- Provide steps to reproduce
- Include any error messages or console output

### Suggesting Features
- Open an issue describing the feature
- Explain the use case and why it would be valuable
- Be open to discussion about implementation approaches

### Pull Requests
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test --workspace`)
5. Commit with clear messages
6. Push to your fork
7. Open a Pull Request

### Code Style
- Rust: Follow standard Rust conventions, run `cargo clippy`
- TypeScript: Use TypeScript strict mode, no `any` types
- Commits: Use conventional commit messages (`feat:`, `fix:`, `docs:`, etc.)

## Areas We Need Help

- **Linux Support**: ALSA/JACK audio backend
- **Windows Support**: WASAPI audio backend
- **Hardware Testing**: Test with different synthesizers and interfaces
- **Documentation**: Tutorials, examples, translations

## License

By contributing, you agree that your contributions will be licensed under the AGPL-3.0 license.
```

**Commit:**

```bash
git add CONTRIBUTING.md
git commit -m "docs: add CONTRIBUTING guide"
```

---

## Phase 5: Final Verification & Release

---

### Task 16: Run Full Verification Suite

**Step 1: Clean build**

```bash
cd /Users/david/Github/batcherbird
cargo clean
cd crates/batcherbird-gui && rm -rf node_modules dist
npm ci
```

**Step 2: Run all checks**

```bash
# Rust
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Frontend
cd crates/batcherbird-gui
npx tsc --noEmit
npm run build

# Full app build
npm run tauri build
```

**Step 3: Verify DMG exists and is reasonable size**

```bash
ls -lh crates/batcherbird-gui/src-tauri/target/release/bundle/dmg/*.dmg
```

---

### Task 17: Create Release Commit

**Step 1: Update version if needed**

Check `crates/batcherbird-gui/src-tauri/tauri.conf.json` has `"version": "0.1.0"`

**Step 2: Final commit**

```bash
git add -A
git status
git commit -m "chore: prepare v0.1.0 release"
```

**Step 3: Create tag**

```bash
git tag -a v0.1.0 -m "BatcherBird v0.1.0 - Initial public release"
```

---

### Task 18: Push and Verify CI

**Step 1: Push to remote**

```bash
git push origin main
git push origin v0.1.0
```

**Step 2: Monitor GitHub Actions**

- Check CI workflow passes
- Check Release workflow creates draft release
- Download and test the DMG artifact

**Step 3: Publish release**

- Go to GitHub Releases
- Edit the draft release
- Review auto-generated release notes
- Publish

---

## Summary: Execution Order

| Phase | Tasks | Approach |
|-------|-------|----------|
| **1. Discovery** | Tasks 1-5 | Run in parallel with sub-agents |
| **2. Remediation** | Tasks 6-8 | Sequential, based on findings |
| **3. Infrastructure** | Tasks 9-11 | Sequential |
| **4. Documentation** | Tasks 12-15 | Can parallelize some |
| **5. Release** | Tasks 16-18 | Sequential, verification-heavy |

**Estimated task count:** 18 tasks
**Parallelizable:** Phase 1 (5 tasks), some of Phase 4

---

## Files to Create

| File | Purpose |
|------|---------|
| `.github/workflows/ci.yml` | CI pipeline |
| `.github/workflows/release.yml` | Release automation |
| `CHANGELOG.md` | Version history |
| `CONTRIBUTING.md` | Contributor guide |
| `docs/reports/code-review-rust.md` | Findings |
| `docs/reports/code-review-frontend.md` | Findings |
| `docs/reports/security-audit.md` | Findings |
| `docs/reports/static-analysis-rust.md` | Findings |
| `docs/reports/static-analysis-frontend.md` | Findings |
| `docs/KNOWN_ISSUES.md` | Deferred issues |

## Files to Modify

| File | Changes |
|------|---------|
| `README.md` | URLs, limitations, troubleshooting |
| `LICENSE` | Already AGPL-3.0 ✓ |
| `Cargo.toml` | Add license/repo to workspace |
| `crates/*/Cargo.toml` | Reference workspace license |
| `crates/batcherbird-gui/package.json` | License field |
