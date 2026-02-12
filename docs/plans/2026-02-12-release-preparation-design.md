# BatcherBird v0.1.0 Release Preparation Plan

**Date:** 2026-02-12
**Goal:** Public GitHub release with clean repo, good README, downloadable macOS binary
**Constraint:** No MIDI hardware available for testing

---

## Context

BatcherBird was created to solve the tedium of manually sampling hardware synthesizers. The core motivation:
- **Problem:** Recording 60+ notes across velocity layers manually takes hours of repetitive work
- **Solution:** Automate the tedious parts so musicians can focus on music
- **Vision:** Open source so others benefit

### Current State
- Epics 1-3 completed (professional audio quality, real-time visualization, seamless workflow)
- Epic 4 (advanced sampling features) defined but not started
- Core recording and export features exist but haven't been battle-tested end-to-end
- No hardware available for testing currently

### What "Release Ready" Means
- Clean, reviewable code
- Security-checked (no secrets, no obvious vulnerabilities)
- Working build that produces downloadable `.dmg`
- Honest README about current state and limitations
- AGPL-3.0 license

### What It Does NOT Require
- Feature completeness (Epic 4 can wait)
- Full test coverage
- Cross-platform support
- Perfect polish

---

## Work Streams

### 1. Code Review & Cleanup

**Goal:** Codebase is clean enough that a stranger could clone it and not cringe.

**Areas to review:**

| Area | What to look for |
|------|------------------|
| Dead code | Unused functions, commented-out blocks, orphaned files |
| Debug artifacts | `console.log`, `println!`, `dbg!`, hardcoded test paths |
| TODO/FIXME | Fix them, remove them, or convert to GitHub issues |
| Code consistency | Naming conventions, file organization, import patterns |
| Hardcoded values | Paths like `/Users/david/...`, test credentials, magic numbers |

**Locations to review:**
- `crates/batcherbird-core/` - Rust audio engine
- `crates/batcherbird-gui/src-tauri/` - Tauri backend
- `crates/batcherbird-gui/src/` - React frontend
- `crates/batcherbird-cli/` - CLI (minimal, may remove if unused)

**Deliverable:** Findings report with prioritized cleanup tasks

---

### 2. Security Audit

**Goal:** Safe for public consumption - no secrets exposed, no dangerous vulnerabilities.

**Checks to perform:**

| Area | What to look for |
|------|------------------|
| Git history | Secrets/credentials ever committed (even if later removed) |
| Environment files | `.env` files, API keys, tokens in code |
| Dependencies | Known vulnerabilities in Rust crates and npm packages |
| File system access | Path traversal risks, unsafe file operations |
| IPC security | Tauri command exposure, input validation on frontend-backend calls |
| Permissions | What capabilities does the app request? Are they all necessary? |

**Tools:**
- `cargo audit` - Rust dependency vulnerabilities
- `npm audit` - Node dependency vulnerabilities
- `git log` / `git grep` - Search history for secrets
- Manual review of `tauri.conf.json` capabilities

**Special attention:**
- Tauri's webview exposure configuration
- File paths from user input
- MIDI/Audio device access patterns

**Deliverable:** Security findings report, remediation for any issues found

---

### 3. Bug Discovery & Fixing

**Goal:** Find and fix bugs that would embarrass us or block basic usage.

**Methods:**

| Phase | Method |
|-------|--------|
| Static analysis | Rust compiler warnings, `clippy` lints, TypeScript strict mode |
| Logic review | Read through core workflows looking for edge cases |
| Error handling | Find panics, unwraps, missing error states |
| UI states | Check for impossible states, missing loading/error UI |

**Priority tiers:**

| Tier | Fix before release? | Examples |
|------|---------------------|----------|
| P0 - Blockers | Yes | Crashes, data loss, security issues |
| P1 - Embarrassing | Yes | Obvious broken UI, misleading errors |
| P2 - Annoying | If time allows | Minor glitches, cosmetic issues |
| P3 - Nice to have | No, track as issues | Improvements, edge cases |

**What we won't do:**
- Chase bugs that require hardware to reproduce
- Add new features disguised as "bug fixes"
- Refactor working code for aesthetics

**Deliverable:** Bug list with priorities, fixes for P0/P1 issues

---

### 4. Build & Release Pipeline

**Goal:** Anyone can download a working `.dmg` from GitHub Releases.

**Components to set up:**

| Component | Purpose |
|-----------|---------|
| GitHub Actions CI | Run on every push: build, lint, test |
| Release workflow | Triggered on version tag, builds macOS `.dmg` |
| Version tagging | Semantic versioning (v0.1.0 for first public release) |

**CI checks:**
```bash
cargo check          # Rust compiles
cargo clippy         # Lint warnings
cargo test           # Unit tests pass
npm run build        # Frontend builds
cargo tauri build    # Full app builds
```

**Release artifacts:**
- `BatcherBird-0.1.0-macos.dmg` - macOS installer
- Source code archives (automatic from GitHub)

**Deferred:**
- Windows/Linux builds (macOS only for v0.1.0)
- Code signing/notarization (users can right-click → Open)
- Auto-update mechanism

**Deliverable:** Working CI/CD pipeline, successful test release

---

### 5. Documentation

**Goal:** Someone landing on the repo knows what this is, how to use it, and what to expect.

**Documents to create/update:**

| Document | Purpose | Action |
|----------|---------|--------|
| README.md | First impression, features, quick start | Review & update |
| CHANGELOG.md | What's in v0.1.0 | Create |
| CONTRIBUTING.md | How to contribute | Create (simple) |
| LICENSE | AGPL-3.0 | Replace (currently MIT) |

**README updates needed:**
- Replace placeholder URLs (`yourusername/batcherbird` → real repo)
- Update screenshots if UI has changed
- Add "Current Limitations" section
- Verify installation instructions work
- Add troubleshooting (Gatekeeper, permissions)

**License change to AGPL-3.0:**
- Replace LICENSE file content
- Update references in README, Cargo.toml, package.json
- Consider SPDX headers in source files

**Honest messaging to include:**
> "BatcherBird is in early release. Core recording and export features work, but extended sampling sessions haven't been battle-tested. We'd love your feedback and bug reports."

**Deliverable:** Updated documentation, AGPL-3.0 license in place

---

### 6. Testing (Without Hardware)

**Goal:** Validate as much as possible without MIDI gear connected.

**What we can test:**

| Area | Method |
|------|--------|
| Audio processing logic | Unit tests with synthetic audio buffers |
| Sample detection algorithms | Feed known waveforms, verify trim points |
| Loop detection | Test FFT analysis with crafted audio |
| Export formats | Generate DecentSampler/SFZ files, validate XML/syntax |
| File operations | WAV writing, folder creation, naming conventions |
| UI states | Component rendering, state transitions |
| Error handling | Simulate failures, verify graceful degradation |

**What we can't test (document as known limitations):**
- Actual MIDI timing with hardware
- Real-world audio interface behavior
- Full sampling sessions
- Hardware-specific quirks

**Validation approach:**
- Create test audio files (sine waves, decaying tones)
- Run through processing pipeline
- Verify outputs load in DecentSampler (free download)

**Deliverable:** Test suite for non-hardware functionality, documented limitations

---

## Execution Approach

### Phase 1: Discovery (Parallel)
Run code review, security audit, and bug discovery in parallel using sub-agents.
Each produces a findings report.

### Phase 2: Remediation
Prioritize and fix issues found in Phase 1.
Focus on P0/P1 issues only.

### Phase 3: Infrastructure
Set up CI/CD pipeline and release workflow.
Verify builds work.

### Phase 4: Documentation & Polish
Update all documentation.
Change license to AGPL-3.0.
Final review.

### Phase 5: Release
Tag v0.1.0.
Publish release.
Announce.

---

## Success Criteria

- [ ] Clean `cargo clippy` and `npm run lint` output
- [ ] No secrets in codebase or git history
- [ ] No critical/high severity dependency vulnerabilities
- [ ] CI pipeline passes on main branch
- [ ] Release workflow produces working `.dmg`
- [ ] README accurately describes current capabilities and limitations
- [ ] LICENSE is AGPL-3.0
- [ ] CHANGELOG documents v0.1.0 features
- [ ] Export formats (DecentSampler, SFZ) validated to load correctly

---

## Out of Scope for v0.1.0

- Epic 4 features (velocity layer improvements, templates, etc.)
- Windows/Linux support
- Code signing / notarization
- Full test coverage
- Hardware-dependent bug fixes
- New features of any kind
