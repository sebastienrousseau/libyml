<!-- SPDX-License-Identifier: Apache-2.0 OR MIT -->

<p align="center">
  <img src="https://cloudcdn.pro/libyml/v1/logos/libyml.svg" alt="libyml logo" width="128" />
</p>

<h1 align="center">libyml</h1>

<p align="center">
  Deprecated low-level YAML library for Rust. The <code>0.0.6</code>
  release is a thin compatibility shim that forwards every call to
  the upstream <code>unsafe-libyaml</code> so existing call sites
  keep working while you migrate to a maintained alternative of
  your choice.
</p>

<p align="center">
  <a href="https://crates.io/crates/libyml"><img src="https://img.shields.io/crates/v/libyml.svg?style=for-the-badge&color=red&label=deprecated&logo=rust" alt="Crates.io (deprecated)" /></a>
  <a href="https://docs.rs/libyml"><img src="https://img.shields.io/badge/docs.rs-libyml-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="Docs.rs" /></a>
  <a href="./MIGRATION.md"><img src="https://img.shields.io/badge/migration-guide-66c2a5?style=for-the-badge" alt="Migration guide" /></a>
</p>

---

## Contents

**Getting started**

- [Install](#install) — stop-gap shim usage
- [Security: RUSTSEC-2025-0067 fixed in 0.0.6](#security-rustsec-2025-0067-fixed-in-006)
- [Quick Start](#quick-start) — shim usage in twenty lines

**Choosing a replacement**

- [Maintained alternatives](#maintained-alternatives) — picking a destination crate
- [One-minute migration paths](#one-minute-migration-paths) — diff snippets per destination

**Deprecation reference**

- [What changed in 0.0.6](#what-changed-in-006) — the shim, in one paragraph
- [What still works in 0.0.6](#what-still-works-in-006) — surviving tests and examples
- [What was removed in 0.0.6](#what-was-removed-in-006) — the hand-translated C copy
- [Behavioural notes](#behavioural-notes) — three intentional deltas worth knowing

**Operational**

- [MSRV](#msrv) — Rust 1.56.0 floor
- [Documentation](#documentation) — migration guide, alternative-crate docs
- [License](#license)

---

## Install

`libyml = "0.0.6"` is a stop-gap so an in-flight migration doesn't
block your release. Existing call sites compile unchanged; the
compiler emits a deprecation warning at each `use libyml::*` import
pointing at the migration guide.

```toml
[dependencies]
libyml = "0.0.6"
```

The shim itself depends on `unsafe-libyaml` for its implementation
— the upstream `libyml` was originally forked from. The 18 000+
lines of hand-translated C-libyaml code that previous releases
shipped are no longer in the source tree; downstream users link
the upstream's audited copy through this re-export. Whether your
eventual destination is `unsafe-libyaml`, `yaml-rust2`, or
`noyalib` is your call.

---

## Security: RUSTSEC-2025-0067 fixed in 0.0.6

[**RUSTSEC-2025-0067**](https://rustsec.org/advisories/RUSTSEC-2025-0067.html)
flagged **all `libyml` versions ≤ 0.0.5** as unsound — the
`libyml::string::yaml_string_extend` function had a code path
that could trigger undefined behaviour.

**Upgrading to `libyml = "0.0.6"` removes the vulnerable
surface entirely:**

- The entire `libyml::string` module — along with the rest of
  the hand-translated C-libyaml copy — is **gone** from the
  source tree.
- Every public function is now a re-export from the upstream
  `unsafe-libyaml` crate, which has an actively-maintained,
  independently-audited translation of the C parser.
- The `yaml_string_extend` symbol no longer exists in this crate
  at any path. Code that depended on it won't compile — which is
  exactly the desired outcome.

Verification:

```bash
$ cargo update -p libyml --precise 0.0.6
$ grep -r yaml_string_extend $(cargo metadata --format-version 1 \
    | jq -r '.packages[] | select(.name=="libyml") | .manifest_path | rtrimstr("/Cargo.toml")')/src
# (no output — the function is no longer in the source tree)
```

The same structural fix flows through to any
[maintained alternative](#maintained-alternatives) you eventually
migrate to.

### `cargo audit` will still warn — here's why and how to handle it

The RustSec advisory database has chosen **not** to mark `0.0.6`
as patched. The decision was made on the basis that `libyml` is
unmaintained regardless of which version you pick — a position
that applies to the *crate* as a whole rather than to a specific
*code path*. Practical consequence: `cargo audit` and `cargo deny`
will emit RUSTSEC-2025-0067 against `libyml = "0.0.6"` even though
the unsound surface no longer exists in this release.

This repo's own CI suppresses the warning via [`.cargo/audit.toml`](./.cargo/audit.toml)
and [`deny.toml`](./deny.toml). If you're a downstream user who
wants the same suppression in your own project, copy the snippet
below — and feel free to remove it whenever you migrate fully.

```toml
# .cargo/audit.toml — at the workspace root
[advisories]
# RUSTSEC-2025-0067 affects libyml's `yaml_string_extend` helper.
# The 0.0.6 deprecation shim removes that surface entirely; verify
# locally with `grep -r yaml_string_extend src/` (no matches).
# The advisory database tracks the crate's unmaintained status,
# not code presence — so we ignore the advisory ourselves and
# document the structural fix here.
ignore = ["RUSTSEC-2025-0067"]
```

For `cargo deny`, add the same `RUSTSEC-2025-0067` ID to your
`deny.toml`'s `[advisories] ignore` list with the same rationale.

The cleanest long-term path is still to migrate off `libyml`
entirely — the maintained alternatives are listed above.

---

## Quick Start

```rust
#![allow(deprecated)]
use core::mem::MaybeUninit;
use libyml::success::is_success;
use libyml::{
    yaml_parser_delete, yaml_parser_initialize, yaml_parser_parse,
    yaml_parser_set_input_string, YamlEventT, YamlParserT,
    YamlStreamEndEvent,
};

fn main() {
    let yaml = b"name: myapp\nport: 8080\n";
    unsafe {
        let mut parser = MaybeUninit::<YamlParserT>::uninit();
        assert!(is_success(yaml_parser_initialize(parser.as_mut_ptr()).ok));
        let mut parser = parser.assume_init();
        yaml_parser_set_input_string(&mut parser, yaml.as_ptr(), yaml.len() as u64);

        loop {
            let mut event = MaybeUninit::<YamlEventT>::uninit();
            assert!(is_success(yaml_parser_parse(&mut parser, event.as_mut_ptr()).ok));
            let event = event.assume_init();
            if event.type_ == YamlStreamEndEvent { break; }
        }
        yaml_parser_delete(&mut parser);
    }
}
```

Run the bundled examples with `cargo run --example example` (parse
+ emit demo) or `cargo run --example migration` (single-file shim
demo).

---

## Maintained alternatives

Three crates are realistic destinations for a `libyml` user. None
is being prescribed — pick the one that fits the codebase.

| Crate | Latest | Migration shape | Best fit |
| :--- | :--- | :--- | :--- |
| **[`unsafe-libyaml`](https://crates.io/crates/unsafe-libyaml)** | 0.2 | Drop-in upstream — rename PascalCase types/consts to snake_case / SCREAMING_SNAKE_CASE | Codebases that want to stay on the raw libyaml-shaped FFI API on a maintained backend |
| **[`yaml-rust2`](https://crates.io/crates/yaml-rust2)** | 0.9 | Not FFI-shaped — `YamlLoader::load_from_str` returns a `Yaml` AST | Users who want to drop the C-libyaml model entirely while keeping a low-level parser primitive in pure Rust |
| **[`noyalib`](https://crates.io/crates/noyalib)** | 0.0 | Higher-level typed API (`from_str::<T>` / `Value`); pure-Rust, `#![forbid(unsafe_code)]` | Users who can move from event-stream parsing to typed deserialisation — usually the cleanest end-state |

### Decision guide

- **You used `yaml_parser_*` / `yaml_emitter_*` and want to keep
  the same shape** — pick **`unsafe-libyaml`**. Same functions,
  PascalCase → snake_case rename on types and event/style
  constants. The shim is already pulling it in transitively, so
  switching is a flat cost.
- **You want to leave the C model behind but stay low-level** —
  pick **`yaml-rust2`**. Pure-Rust parser, no FFI shapes,
  produces a `Yaml` enum AST.
- **You can move to typed deserialisation** — pick **`noyalib`**.
  Modern serde-integrated API, no `unsafe`, configurable parser
  limits and YAML 1.2 strict resolution. Best long-term landing
  spot for any code that was using `libyml` to back a config
  loader or document model.

---

## One-minute migration paths

Side-by-side diff snippets for each destination. The full
function-mapping tables are in [`MIGRATION.md`](./MIGRATION.md).

### → `unsafe-libyaml`

```diff
-[dependencies]
-libyml = "0.0"
+[dependencies]
+unsafe-libyaml = "0.2"
```

```diff
-use libyml::{
-    yaml_parser_initialize, YamlParserT, YamlUtf8Encoding,
-    YamlPlainScalarStyle,
-};
+use unsafe_libyaml::{
+    yaml_parser_initialize, yaml_parser_t as YamlParserT,
+    YAML_UTF8_ENCODING, YAML_PLAIN_SCALAR_STYLE,
+};
```

Functions keep the same names. Types rename from PascalCase
(`YamlParserT`) to snake_case (`yaml_parser_t`). Enum variants
rename from PascalCase (`YamlUtf8Encoding`) to
SCREAMING_SNAKE_CASE (`YAML_UTF8_ENCODING`). Boolean arguments
change from `c_int` (`0`/`1`) to Rust `bool` (`false`/`true`).

### → `yaml-rust2`

```diff
-[dependencies]
-libyml = "0.0"
+[dependencies]
+yaml-rust2 = "0.9"
```

```diff
-// Event-stream loop with yaml_parser_parse(...)
+use yaml_rust2::YamlLoader;
+let docs = YamlLoader::load_from_str(yaml_str)?;
+let v = &docs[0];
```

`yaml-rust2` returns a `Yaml` enum AST instead of streaming
events. Code that walked the libyml event stream needs a
restructure to walk the AST. This is the right choice when you
want pure-Rust parser primitives without the C-libyaml model.

### → `noyalib`

```diff
-[dependencies]
-libyml = "0.0"
+[dependencies]
+noyalib = "0.0.5"
```

```diff
-// Manual event-stream walk to read keys
+use noyalib::{from_str, Value};
+let cfg: MyConfig = from_str(yaml_str)?;
+// or, untyped:
+let v: Value = from_str(yaml_str)?;
```

If your `libyml` usage was indirect — backing a config loader,
RPC payload codec, or document model — `noyalib` is the cleanest
end-state. Pure-Rust, `#![forbid(unsafe_code)]`, YAML 1.2 strict
resolver, configurable parser limits.

---

## What changed in 0.0.6

`libyml 0.0.6` is a thin compatibility shim. The hand-translated
copy of C `libyaml` that previous releases shipped — ~18 000 lines
across `api.rs`, `scanner.rs`, `parser.rs`, `emitter.rs`,
`dumper.rs`, `loader.rs`, and friends — has been deleted. Every
public function is now re-exported from `unsafe-libyaml` (the
upstream `libyml` was originally forked from); the historical
PascalCase type aliases (`YamlParserT`, `YamlEventT`, …) and the
common PascalCase enum-variant constants (`YamlUtf8Encoding`,
`YamlPlainScalarStyle`, …) are restored on top so existing call
sites compile unchanged.

The shim being backed by `unsafe-libyaml` internally is an
implementation detail, not a recommendation to use
`unsafe-libyaml` specifically. The
[Maintained alternatives](#maintained-alternatives) section above
covers the choice.

---

## What still works in 0.0.6

The shim is wire-compatible with typical user code. The original
test files from `libyml ≤ 0.0.5` are kept under `tests/` (with
small adaptation comments noting the patches) so users can see
their own code's migration shape side-by-side. Verified by
`cargo test --all-targets` + `cargo run --example example` +
`cargo run --example migration`:

| Surface | Status |
| :--- | :--- |
| `tests/test_lib.rs` — retained from 0.0.5, two-line patch (`is_success(call)` → `is_success(call.ok)`, drop `#![no_std]`) | **5 / 5 pass** |
| `tests/test_decode.rs` — retained from 0.0.5 **verbatim** — the `libyml::decode::*` path module re-exports through the shim | **8 / 8 pass** |
| `tests/shim.rs` — new smoke suite: parser init/delete, parse-first-event, emit a `{greeting: hello}` mapping round-trip, type-alias resolution, `success` helpers | **5 / 5 pass** |
| `examples/example.rs` — retained 0.0.5 aggregator shape — runs `examples/apis/main.rs` then a parse + emit demo | **exits 0** |
| `examples/apis/main.rs` — retained from 0.0.5; the `memory` + `string` slabs are kept as comments with Rust-native replacements | **exits 0** |
| `examples/migration.rs` — single-file shim demo | **exits 0** |

The full per-file inventory of retained / patched / removed tests
and examples is in [`MIGRATION.md` § "Test and example coverage in
0.0.6"](./MIGRATION.md#test-and-example-coverage-in-006).

---

## What was removed in 0.0.6

The deep internal modules that previous versions exposed leaked
implementation details of the hand-translated C copy. They are
**removed** in this shim. The right replacement depends on which
alternative you picked:

| Removed from `libyml` | What it was | Where it goes |
| :--- | :--- | :--- |
| `libyml::api` | High-level wrappers over the parser/emitter init/free pairs | Bare functions at `unsafe_libyaml::yaml_parser_*` / `yaml_emitter_*` (re-exported by this shim at the crate root) |
| `libyml::dumper` | `yaml_emitter_open` / `_close` / `_dump` helpers | `unsafe_libyaml::yaml_emitter_open` / `_close` / `_dump` (re-exported at the crate root) |
| `libyml::decode` | `yaml_parser_initialize` / `_delete` wrappers | `unsafe_libyaml::yaml_parser_initialize` / `_delete` (re-exported at the crate root) |
| `libyml::document` | `yaml_document_*` event/node helpers | `unsafe_libyaml::yaml_document_*` (some are re-exported at the crate root; the rest are reachable via the upstream path) |
| `libyml::loader` | `yaml_parser_load` | `unsafe_libyaml::yaml_parser_load` (re-exported at the crate root) |
| `libyml::memory` | `yaml_malloc` / `yaml_free` / `yaml_realloc` / `yaml_strdup` | None — the upstream uses Rust's `alloc` directly; allocate with Rust's standard primitives instead |
| `libyml::string` | `yaml_string_extend` / `_join` helpers | None — internal helpers of the C copy; rewrite using Rust's `Vec`/`String` |
| `libyml::yaml` | Public path-form re-exports of all enums + structs | Type aliases at the `libyml` crate root (`YamlParserT`, etc.); upstream snake_case names available under `unsafe_libyaml::` |
| `libyml::success::Success` (as a nameable type) | `#[derive(PartialEq, Debug)]` struct wrapping `bool` | Read `.ok` on the upstream's return value directly; the shim retains `is_success(bool)` and `is_failure(bool)` helpers |

The full table is in [`MIGRATION.md`](./MIGRATION.md#removed-in-006).

---

## Behavioural notes

The shim is backed by `unsafe-libyaml`'s upstream code, which has
diverged from the fork's snapshot in three user-visible ways:

1. **Boolean parameters take `bool`, not `c_int`.** Previously
   `yaml_scalar_event_initialize(..., 1, 1, style)` compiled with
   `c_int` arguments. Under the shim the function signature comes
   from `unsafe-libyaml`, so the same call site needs `true` /
   `false` instead of `1` / `0`. This is a hard compile error,
   not a silent change.

2. **`Success` is no longer a nameable type.** The upstream keeps
   its `Success` struct in a private module — the value still
   flows out of `yaml_*` calls and you can still read `.ok`, but
   you can no longer write `fn foo() -> libyml::success::Success`.
   The retained `is_success` / `is_failure` helpers now take
   `bool` directly, so chain them as `is_success(call(...).ok)`.

3. **Enum variants rename PascalCase → SCREAMING_SNAKE_CASE in
   `match` arms.** The shim defines `pub const YamlUtf8Encoding`
   etc. so the names still work in **value position**
   (`yaml_emitter_set_encoding(emit, YamlUtf8Encoding)`). In
   refutable **patterns** (`match enc { YamlUtf8Encoding => … }`)
   the upstream's SCREAMING_SNAKE_CASE name is required
   (`YAML_UTF8_ENCODING`). Both spellings are re-exported.

The full mapping is in [`MIGRATION.md`](./MIGRATION.md#behavioural-notes).

---

## MSRV

`libyml 0.0.6` requires **Rust 1.56.0** — the same floor as
`unsafe-libyaml`. The previous releases also required 1.56, so
this is not a bump. Users on older toolchains should pin
`libyml = "=0.0.5"` until they can move forward.

---

## Documentation

| Document | Covers |
| --- | --- |
| [`MIGRATION.md`](./MIGRATION.md) | Find/replace tables per destination, full removed-surface mapping, test/example coverage triage |
| [`unsafe-libyaml`](https://docs.rs/unsafe-libyaml) — [GitHub](https://github.com/dtolnay/unsafe-libyaml) | Upstream destination — same FFI shape, maintained |
| [`yaml-rust2`](https://docs.rs/yaml-rust2) | Pure-Rust low-level parser destination |
| [`noyalib`](https://docs.rs/noyalib) — [GitHub](https://github.com/sebastienrousseau/noyalib) | Modern pure-Rust typed-API destination |
| [docs.rs/libyml](https://docs.rs/libyml) | API reference for this shim — every item carries the `#[deprecated]` banner |

---

## License

Dual-licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) or [MIT](https://opensource.org/licenses/MIT), at your option.

<p align="right"><a href="#contents">Back to Top</a></p>
