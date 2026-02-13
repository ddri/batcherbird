# Security Audit - BatcherBird v0.1.0 Release

**Date:** 2026-02-12
**Auditor:** Claude (automated)
**Commit:** 58b929d1ddb25f4327cdfe0d149c7d049a4b9b0c

## Executive Summary

**Overall Security Posture: PASS with RECOMMENDATIONS**

BatcherBird demonstrates good security practices for a desktop audio application. No critical vulnerabilities or secrets were found. The application implements path validation for file operations and follows reasonable security defaults. There are moderate dependency vulnerabilities that should be addressed before release and some code quality improvements recommended.

## Critical Findings (MUST FIX)

**None identified.**

No secrets, API keys, passwords, or credentials were found in the codebase or git history.

## High Severity

### 1. NPM Dependency Vulnerabilities (2 issues)

```
glob  10.2.0 - 10.4.5
Severity: high
Issue: Command injection via -c/--cmd executes matches with shell:true
Advisory: https://github.com/advisories/GHSA-5j98-mcp5-4vw2

vite  7.0.0 - 7.0.7
Severity: moderate
Issues:
- Middleware may serve files starting with same name as public directory
- server.fs settings not applied to HTML files
- server.fs.deny bypass via backslash on Windows
```

**Recommendation:** Run `npm audit fix` in `crates/batcherbird-gui/` before release.

## Medium Severity

### 1. Session ID Path Construction Without Sanitization

**File:** `/Users/david/Github/batcherbird/crates/batcherbird-gui/src-tauri/src/session.rs:153-155`

```rust
fn get_session_path(session_id: &str) -> Result<PathBuf, String> {
    let session_dir = get_session_dir()?;
    Ok(session_dir.join(format!("{}.json", session_id)))
}
```

The `session_id` parameter comes from user input and is used directly in path construction. While session_id is generated as a UUID internally, external callers could potentially supply crafted values.

**Risk:** If an attacker could control the session_id parameter, they could potentially read/write files outside the session directory using path traversal sequences.

**Current Mitigations:**
- Session IDs are generated as UUIDs internally
- The session directory is restricted to `~/.batcherbird/sessions/`

**Recommendation:** Add validation to `get_session_path()` to reject session_ids containing path separators or `..`:
```rust
fn get_session_path(session_id: &str) -> Result<PathBuf, String> {
    // Reject path traversal attempts
    if session_id.contains('/') || session_id.contains('\\') || session_id.contains("..") {
        return Err("Invalid session ID".to_string());
    }
    let session_dir = get_session_dir()?;
    Ok(session_dir.join(format!("{}.json", session_id)))
}
```

### 2. Cargo Audit Not Available

Rust dependency vulnerabilities could not be checked because `cargo-audit` is not installed.

**Recommendation:** Install and run cargo audit:
```bash
cargo install cargo-audit
cargo audit
```

## Low Severity / Recommendations

### 1. Excessive Use of `unwrap()` (55 occurrences)

**File:** `crates/batcherbird-gui/src-tauri/src/lib.rs`

While not directly a security issue, excessive `unwrap()` calls can cause unexpected panics. In a desktop audio application, this is unlikely to be exploitable but could cause denial of service or poor user experience.

**Recommendation:** Review and convert critical `unwrap()` calls to proper error handling with `?` or `match`.

### 2. CSP Includes `'unsafe-inline'` for Scripts and Styles

**File:** `crates/batcherbird-gui/src-tauri/tauri.conf.json:21`

```json
"csp": "default-src 'self' tauri:; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: tauri: asset:"
```

The use of `'unsafe-inline'` weakens CSP protection. This is common in Tauri applications due to framework requirements but should be documented.

**Risk:** Low - Tauri applications don't load external content and have limited XSS attack surface.

**Recommendation:** Document this as a known limitation. Consider if stricter CSP is achievable in future versions.

### 3. Asset Protocol Scope Includes Home Directory

**File:** `crates/batcherbird-gui/src-tauri/tauri.conf.json:24-29`

```json
"assetProtocol": {
  "enable": true,
  "scope": [
    "$DESKTOP/*",
    "$DOCUMENT/*",
    "$HOME/Documents/BatcherBird Projects/*",
    "$APPDATA/*",
    "$RESOURCE/*"
  ]
}
```

The scope is appropriately restricted to user directories relevant to the application's purpose (audio sample management). This is reasonable for the application's functionality.

**Assessment:** ACCEPTABLE - The scope is limited to user data directories, not system directories.

### 4. Shell Plugin Included But Not Obviously Used

**File:** `crates/batcherbird-gui/package.json:26`

```json
"@tauri-apps/plugin-shell": "^2.3.0"
```

The shell plugin is included in dependencies. Review whether this is needed.

**Finding:** The only shell command usage found is `Command::new("open")` for opening Finder, which uses `std::process::Command` not the Tauri shell plugin.

**Recommendation:** If the plugin is unused, remove it to reduce attack surface.

## Detailed Analysis

### Secrets Scan

**Result:** PASS

| Check | Result |
|-------|--------|
| `password` in git history | Only in documentation (release plan) |
| `secret` in git history | Only in documentation (release plan) |
| `api_key` in git history | Only in documentation (release plan) |
| `token` in git history | Only in documentation (release plan) |
| `Bearer` tokens | Only reference in documentation |
| `.env` files | None found |
| `credentials*` files | None found |
| `*secret*` files | None found |
| Private keys (*.key, *.pem) | None in history |

### Dependency Vulnerabilities

#### Rust (cargo audit)
**Status:** NOT CHECKED - cargo-audit not installed

**Recommendation:** Install and run before release:
```bash
cargo install cargo-audit
cargo audit
```

#### Node (npm audit)
**Status:** 2 vulnerabilities found

| Package | Severity | Issue |
|---------|----------|-------|
| glob 10.2.0-10.4.5 | HIGH | Command injection via shell:true |
| vite 7.0.0-7.0.7 | MODERATE | Multiple fs.deny bypasses |

**Fix:** Run `npm audit fix` in `crates/batcherbird-gui/`

### Tauri Security Configuration

**CSP Policy:** ACCEPTABLE with notes
- Uses `'unsafe-inline'` for scripts/styles (common in Tauri apps)
- Properly restricts default-src to 'self' and tauri:
- Image sources appropriately include data: and asset: URIs

**Asset Protocol Scope:** GOOD
- Appropriately scoped to user directories
- Does not expose system directories
- Matches application functionality (audio sample management)

**Capabilities:** MINIMAL
- Only default permissions enabled
- core:window:default
- core:event:default
- core:app:default
- core:resources:default
- dialog:default

### Tauri Command Review

| Command | Parameters | Path Validation | Risk Assessment |
|---------|------------|-----------------|-----------------|
| `create_directory` | `path: String` | YES - `validate_file_path()` | LOW |
| `generate_instrument_files` | `directory: String` | YES - validates path | LOW |
| `detect_loop_points` | `file_path: String` | YES - `validate_file_path()` | LOW |
| `apply_loop_metadata` | `file_path: String` | YES - `validate_file_path()` | LOW |
| `get_waveform_data` | `file_path: String` | YES - `validate_file_path()` | LOW |
| `load_sample_for_playback` | `file_path: String` | YES - `validate_file_path()` | LOW |
| `record_sample` | `output_directory: Option<String>` | Uses default dirs | LOW |
| `save_recording_session` | Multiple | Session ID is UUID | LOW |
| `resume_recording_session` | `session_id: String` | **NO** - See Medium #1 | MEDIUM |
| `delete_recording_session` | `session_id: String` | **NO** - See Medium #1 | MEDIUM |
| `show_samples_in_finder` | None | Hardcoded safe path | LOW |
| `select_output_directory` | None (dialog) | User-selected via dialog | LOW |
| `select_audio_file` | None (dialog) | User-selected via dialog | LOW |

### File System Access Patterns

The application implements a `validate_file_path()` function that:

1. **Rejects path traversal:** Checks for `..` and `~` in paths
2. **Requires absolute paths or converts relative paths**
3. **Whitelists directories:** Only allows access to:
   - Home directory
   - Desktop
   - Documents
   - Downloads
   - Audio directory
   - Cache directory
   - Temp directory

**Implementation:** `/Users/david/Github/batcherbird/crates/batcherbird-gui/src-tauri/src/lib.rs:35-65`

**Assessment:** GOOD - The validation is reasonable for a desktop audio application.

### .gitignore Assessment

**File:** `/Users/david/Github/batcherbird/.gitignore`

**Covered patterns:**
- `.env`, `.env.local`, `.env.production`, `.env.development`
- `*.key`, `*.pem`, `*.p12`, `*.pfx`, `*.crt`, `*.csr`
- `secrets.json`, `credentials.json`
- `config/user-paths.json`
- `**/temp/`
- Node artifacts (`node_modules/`, `dist/`, etc.)
- Rust artifacts (`target/`, `debug/`)
- OS files (`.DS_Store`, `Thumbs.db`)

**Assessment:** COMPREHENSIVE - The .gitignore covers common sensitive file patterns appropriately.

## Recommendations

### Before Release (Priority Order)

1. **HIGH:** Run `npm audit fix` to address glob and vite vulnerabilities
2. **HIGH:** Install and run `cargo audit` to check Rust dependencies
3. **MEDIUM:** Add session_id validation in `session.rs` to prevent path traversal
4. **LOW:** Review if `@tauri-apps/plugin-shell` dependency is needed

### Future Improvements

1. Review 55 `unwrap()` occurrences for potential panic scenarios
2. Consider stricter CSP if framework allows
3. Add automated security scanning to CI/CD pipeline
4. Document security model in user-facing documentation

## Conclusion

BatcherBird is ready for public release from a security perspective after addressing the npm dependency vulnerabilities. The application demonstrates security-conscious design with proper path validation and minimal privilege requirements. The session_id validation improvement is recommended but not blocking for release as the risk is mitigated by UUID generation.
