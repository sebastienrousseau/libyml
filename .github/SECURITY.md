# Security policy

## Status: `libyml` is deprecated

`libyml` is **unmaintained**. The `0.0.6` release is a thin
compatibility shim that forwards every call to a maintained
upstream (`unsafe-libyaml`) so existing call sites keep compiling.
See [`MIGRATION.md`](../MIGRATION.md) for the recommended
migration paths.

## Vulnerable surface — removed in 0.0.6

Previous releases shipped a hand-translated copy of C `libyaml`
(~18 000 lines across `api.rs`, `scanner.rs`, `parser.rs`,
`emitter.rs`, …) under `#![allow(unsafe_code)]`. **The 0.0.6
release deletes that copy entirely** and re-exports the upstream
`unsafe-libyaml` instead. Bug fixes and audits flowing through
the upstream now land in users of this shim on a plain
`cargo update`, without a new `libyml` release.

Verification:

```bash
cargo update -p libyml --precise 0.0.6
cargo tree -p libyml | grep -E 'libyml|unsafe-libyaml'
# → only unsafe-libyaml under libyml; the hand-translated copy is gone
```

## Supported versions

| Version | Status |
| :--- | :--- |
| `0.0.6` | Deprecation shim — backed by the maintained upstream; safe to use as a stop-gap |
| `≤ 0.0.5` | **End-of-life.** Pinning these keeps a stale hand-translated copy of C libyaml in your dependency graph. Migrate. |

## Reporting a vulnerability

Because the crate is unmaintained, please **file new
vulnerability reports against the upstream crate**, not against
`libyml`:

- For findings in the YAML parser/emitter (the actual code path):
  [github.com/dtolnay/unsafe-libyaml/security/advisories/new](https://github.com/dtolnay/unsafe-libyaml/security/advisories/new)
- For findings in the alternative crates listed in
  [`MIGRATION.md`](../MIGRATION.md): use that crate's own
  disclosure channel.

If a finding is **specific to the `libyml` shim layer itself**
(the re-export glue in `src/lib.rs`, not the underlying
parser/emitter), open a private security advisory against this
repository:
<https://github.com/sebastienrousseau/libyml/security/advisories/new>.

When reporting, include:

- Type of issue (e.g. buffer overflow, soundness, SQL injection,
  cross-site scripting)
- Full paths of source file(s) related to the manifestation of
  the issue
- The location of the affected source code (tag/branch/commit or
  direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact, including how an attacker might exploit the issue

This information helps triage your report quickly.
