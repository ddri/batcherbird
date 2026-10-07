# Vizia core accessibility patch

Source: the crates.io `vizia_core` 0.3.0 package (MIT), with its original
`src/`, `resources/`, and normalized `Cargo.toml`. `LICENSE` is copied from
the matching `vizia` 0.3.0 package, copyright George Atkinson.
Only this crate is patched; all other Vizia dependencies remain on crates.io.
Upstream VCS commit: `35171576875d7c3721301f1a5bbe7255605fe9bc`, directory
`crates/vizia_core`. Text line endings are normalized to LF and trailing
whitespace removed for repository checks; other source content is preserved.

Changes are confined to three upstream source files:

- `src/systems/accessibility.rs`: forward `.name()` to AccessKit's `label`
  property; advertise the existing Click handlers of the built-in button,
  toggle button, checkbox, and radio button views when enabled.
- `src/events/event_manager.rs`: reject Click requests for disabled targets
  before dispatch, including requests made against stale accessibility nodes.
- `src/views/list.rs`: replace, rather than accumulate, a bound selection and
  its focused index. This keeps keyboard navigation aligned with the application's
  current sample when entering review or when selection changes externally.

The application integration tests in
`crates/batcherbird-vizia/tests/accessibility.rs` exercise the actual generated
AccessKit tree and dispatch. The vendor crate is excluded from workspace lint
and test membership; it still builds as the patched application dependency.

Remove the Cargo patch and this directory after adopting an upstream release
that forwards labels, advertises button activation, guards disabled clicks, and
replaces bound list selections.
Native VoiceOver testing remains a separate acceptance check.

The vendored manifest also allows only `mismatched_lifetime_syntaxes`, restoring
the quiet dependency build that crates.io's capped lints provided. The upstream
source signatures and the application's strict warnings gate remain unchanged.
