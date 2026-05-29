# Migrating off `libyml`

`libyml` is unmaintained. The `0.0.6` release is a thin
compatibility shim so existing call sites keep working while you
migrate to a maintained alternative.

> ## ⚠️ Security: RUSTSEC-2025-0067 is structurally fixed in 0.0.6
>
> [RUSTSEC-2025-0067](https://rustsec.org/advisories/RUSTSEC-2025-0067.html)
> flagged all `libyml ≤ 0.0.5` as unsound — the
> `libyml::string::yaml_string_extend` function had a code path
> that could trigger undefined behaviour. **Upgrading to
> `libyml = "0.0.6"` removes the vulnerable surface entirely** —
> the entire `libyml::string` module is gone from the source tree
> alongside the rest of the hand-translated C-libyaml copy, and
> every public function is now re-exported from the upstream
> `unsafe-libyaml` crate.
>
> **`cargo audit` will still warn anyway.** The RustSec advisory
> database tracks the crate's unmaintained status across all
> versions and has chosen not to mark `0.0.6` as patched. The
> warning is a maintainer-status signal at this point, not a
> code-presence signal. To suppress it in your own project, copy
> the snippet from the
> [README's "cargo audit" section](./README.md#cargo-audit-will-still-warn--heres-why-and-how-to-handle-it),
> or migrate fully to one of the maintained alternatives below.

The shim itself depends on
[`unsafe-libyaml`](https://crates.io/crates/unsafe-libyaml) — the
upstream Rust translation of C `libyaml` that `libyml` was
originally forked from — for its implementation. That's an
implementation detail, not a recommendation that you must migrate
to `unsafe-libyaml` specifically. Three crates are realistic
destinations; pick the one that fits.

| Destination | Migration shape | When it's the right choice |
| :--- | :--- | :--- |
| **[`unsafe-libyaml`](https://crates.io/crates/unsafe-libyaml)** | Drop-in upstream — rename PascalCase types/consts to snake_case / SCREAMING_SNAKE_CASE | Codebases that want to stay on the raw libyaml-shaped FFI API on a maintained backend |
| **[`yaml-rust2`](https://crates.io/crates/yaml-rust2)** | Not FFI-shaped — `YamlLoader::load_from_str` returns a `Yaml` AST | Users who want to drop the C-libyaml model entirely while keeping a low-level parser primitive in pure Rust |
| **[`noyalib`](https://crates.io/crates/noyalib)** | Higher-level typed API (`from_str::<T>` / `Value`); pure-Rust, `#![forbid(unsafe_code)]` | Users who can move from event-stream parsing to typed deserialisation — usually the cleanest end-state |

The rest of this document describes each migration path, the
public-surface mapping, the modules that are gone in this shim,
and the behavioural deltas to know about.

---

## Path A — Stay on `libyml = "0.0.6"` (stop-gap)

If you cannot migrate right now, depending on the shim keeps your
code compiling. The compiler emits a deprecation warning at every
`use libyml::*` import so you can budget the work.

```toml
[dependencies]
libyml = "0.0.6"
```

No code changes required, with three caveats covered in the
[Behavioural notes](#behavioural-notes) below. Roughly:

- C-int boolean arguments (`1` / `0`) flip to Rust `bool`
  (`true` / `false`) — hard compile error at the call site, easy
  to fix.
- `libyml::success::Success` is no longer a nameable type — read
  `.ok` on the return value directly; the shim's helpers
  (`is_success` / `is_failure`) now take `bool`.
- Enum variants kept their PascalCase names in **value position**
  but rename to SCREAMING_SNAKE_CASE in `match` arms.

---

## Path B — Migrate to `unsafe-libyaml`

```diff
-[dependencies]
-libyml = "0.0"
+[dependencies]
+unsafe-libyaml = "0.2"
```

```diff
-use libyml::{yaml_parser_initialize, YamlParserT, YamlUtf8Encoding};
+use unsafe_libyaml::{
+    yaml_parser_initialize,
+    yaml_parser_t as YamlParserT,
+    YAML_UTF8_ENCODING,
+};
```

Or rename at the import site for a one-line diff:

```rust
use unsafe_libyaml as libyml;
// then update the PascalCase type/const names individually.
```

That is the entire migration for codebases that were using
`libyml` as a literal libyaml-shaped FFI surface. The
public-surface mapping is in [§ Public-surface
mapping](#public-surface-mapping) below.

---

## Path C — Migrate to `yaml-rust2` (pure-Rust low-level)

```diff
-[dependencies]
-libyml = "0.0"
+[dependencies]
+yaml-rust2 = "0.9"
```

```diff
-let mut parser = MaybeUninit::<YamlParserT>::uninit();
-yaml_parser_initialize(parser.as_mut_ptr());
-let mut parser = parser.assume_init();
-yaml_parser_set_input_string(&mut parser, src.as_ptr(), src.len() as u64);
-loop {
-    let mut event = MaybeUninit::<YamlEventT>::uninit();
-    yaml_parser_parse(&mut parser, event.as_mut_ptr());
-    // ... handle event ...
-}
+use yaml_rust2::YamlLoader;
+let docs = YamlLoader::load_from_str(src)?;
+let v = &docs[0];
+// ... walk the Yaml AST ...
```

`yaml-rust2` is a pure-Rust YAML parser — the active continuation
of the original `yaml-rust` crate. It returns a `Yaml` enum (its
own AST), **not a stream of events**. Migrating means restructuring
event-loop code into AST traversal. This is the right choice when
you actually want pure-Rust parser primitives — custom loaders,
lint tools, format-preserving editors — and can drop the C-libyaml
model.

For typed `from_str::<T>` flows, prefer `noyalib`.

---

## Path D — Migrate to `noyalib` (modern typed API)

```diff
-[dependencies]
-libyml = "0.0"
+[dependencies]
+noyalib = "0.0.5"
```

```diff
-// Manual event-stream walk to extract `name` and `port`
-let mut parser = MaybeUninit::<YamlParserT>::uninit();
-yaml_parser_initialize(parser.as_mut_ptr());
-// ... ~30 lines of event dispatch ...
+use noyalib::from_str;
+#[derive(serde::Deserialize)]
+struct Config { name: String, port: u16 }
+let cfg: Config = from_str(yaml_str)?;
```

`noyalib` is a modern, pure-Rust, `#![forbid(unsafe_code)]` YAML
library with a high-level typed API (`from_str::<T>` / `Value`),
configurable parser limits, and YAML 1.2 strict resolution. It
covers the use case where `libyml` was a building block for a
config loader or document model — i.e. exactly the cases where
the event-stream API was incidental rather than essential.

| Surface | `noyalib` mapping |
| :--- | :--- |
| Hand-written event-stream walk for typed extraction | `noyalib::from_str::<T>` |
| `yaml_parser_load` → document tree | `noyalib::from_str::<noyalib::Value>` |
| `yaml_emitter_emit` event-stream emission | `noyalib::to_string(&value)` |
| Anchor / alias resolution (manual `&a` / `*a` walking) | Transparent — `noyalib` resolves anchors during parse |
| Custom tag handling | `Value::Tagged` variant preserved exactly |
| Streaming over large input | `noyalib::Deserializer::from_reader(...)` |

---

## Public-surface mapping

The common surface is preserved name-for-name through the
`libyml 0.0.6` shim, and maps directly to `unsafe-libyaml` for
users taking Path B:

| `libyml` (≤ 0.0.5)                          | `libyml` 0.0.6 shim                       | Direct `unsafe-libyaml` equivalent           |
| ------------------------------------------- | ----------------------------------------- | -------------------------------------------- |
| `libyml::yaml_parser_initialize`            | unchanged                                 | `unsafe_libyaml::yaml_parser_initialize`     |
| `libyml::yaml_parser_delete`                | unchanged                                 | `unsafe_libyaml::yaml_parser_delete`         |
| `libyml::yaml_parser_set_input_string`      | unchanged                                 | `unsafe_libyaml::yaml_parser_set_input_string` |
| `libyml::yaml_parser_set_input`             | unchanged                                 | `unsafe_libyaml::yaml_parser_set_input`      |
| `libyml::yaml_parser_set_encoding`          | unchanged                                 | `unsafe_libyaml::yaml_parser_set_encoding`   |
| `libyml::yaml_parser_parse`                 | unchanged                                 | `unsafe_libyaml::yaml_parser_parse`          |
| `libyml::yaml_parser_scan`                  | unchanged                                 | `unsafe_libyaml::yaml_parser_scan`           |
| `libyml::yaml_parser_load`                  | unchanged                                 | `unsafe_libyaml::yaml_parser_load`           |
| `libyml::yaml_emitter_initialize`           | unchanged                                 | `unsafe_libyaml::yaml_emitter_initialize`    |
| `libyml::yaml_emitter_delete`               | unchanged                                 | `unsafe_libyaml::yaml_emitter_delete`        |
| `libyml::yaml_emitter_set_output`           | unchanged                                 | `unsafe_libyaml::yaml_emitter_set_output`    |
| `libyml::yaml_emitter_set_output_string`    | unchanged                                 | `unsafe_libyaml::yaml_emitter_set_output_string` |
| `libyml::yaml_emitter_set_encoding`         | unchanged                                 | `unsafe_libyaml::yaml_emitter_set_encoding`  |
| `libyml::yaml_emitter_set_canonical`        | unchanged                                 | `unsafe_libyaml::yaml_emitter_set_canonical` |
| `libyml::yaml_emitter_set_indent`           | unchanged                                 | `unsafe_libyaml::yaml_emitter_set_indent`    |
| `libyml::yaml_emitter_set_width`            | unchanged                                 | `unsafe_libyaml::yaml_emitter_set_width`     |
| `libyml::yaml_emitter_set_unicode`          | unchanged                                 | `unsafe_libyaml::yaml_emitter_set_unicode`   |
| `libyml::yaml_emitter_set_break`            | unchanged                                 | `unsafe_libyaml::yaml_emitter_set_break`     |
| `libyml::yaml_emitter_open`                 | unchanged                                 | `unsafe_libyaml::yaml_emitter_open`          |
| `libyml::yaml_emitter_close`                | unchanged                                 | `unsafe_libyaml::yaml_emitter_close`         |
| `libyml::yaml_emitter_dump`                 | unchanged                                 | `unsafe_libyaml::yaml_emitter_dump`          |
| `libyml::yaml_emitter_emit`                 | unchanged                                 | `unsafe_libyaml::yaml_emitter_emit`          |
| `libyml::yaml_emitter_flush`                | unchanged                                 | `unsafe_libyaml::yaml_emitter_flush`         |
| `libyml::yaml_event_delete`                 | unchanged                                 | `unsafe_libyaml::yaml_event_delete`          |
| `libyml::yaml_token_delete`                 | unchanged                                 | `unsafe_libyaml::yaml_token_delete`          |
| `libyml::yaml_*_event_initialize`           | unchanged                                 | `unsafe_libyaml::yaml_*_event_initialize`    |
| `libyml::yaml_document_initialize`          | unchanged                                 | `unsafe_libyaml::yaml_document_initialize`   |
| `libyml::yaml_document_delete`              | unchanged                                 | `unsafe_libyaml::yaml_document_delete`       |
| `libyml::yaml_document_get_root_node`       | unchanged                                 | `unsafe_libyaml::yaml_document_get_root_node`|
| `libyml::yaml_document_get_node`            | unchanged                                 | `unsafe_libyaml::yaml_document_get_node`     |
| `libyml::YamlParserT`                       | unchanged (alias)                         | `unsafe_libyaml::yaml_parser_t`              |
| `libyml::YamlEmitterT`                      | unchanged (alias)                         | `unsafe_libyaml::yaml_emitter_t`             |
| `libyml::YamlEventT`                        | unchanged (alias)                         | `unsafe_libyaml::yaml_event_t`               |
| `libyml::YamlTokenT`                        | unchanged (alias)                         | `unsafe_libyaml::yaml_token_t`               |
| `libyml::YamlDocumentT`                     | unchanged (alias)                         | `unsafe_libyaml::yaml_document_t`            |
| `libyml::YamlNodeT`                         | unchanged (alias)                         | `unsafe_libyaml::yaml_node_t`                |
| `libyml::YamlMarkT`                         | unchanged (alias)                         | `unsafe_libyaml::yaml_mark_t`                |
| `libyml::YamlVersionDirectiveT`             | unchanged (alias)                         | `unsafe_libyaml::yaml_version_directive_t`   |
| `libyml::YamlTagDirectiveT`                 | unchanged (alias)                         | `unsafe_libyaml::yaml_tag_directive_t`       |
| `libyml::YamlUtf8Encoding` (value position) | unchanged (`pub const`)                   | `unsafe_libyaml::YAML_UTF8_ENCODING`         |
| `libyml::YamlPlainScalarStyle` (value)      | unchanged (`pub const`)                   | `unsafe_libyaml::YAML_PLAIN_SCALAR_STYLE`    |
| `libyml::YamlBlockMappingStyle` (value)     | unchanged (`pub const`)                   | `unsafe_libyaml::YAML_BLOCK_MAPPING_STYLE`   |
| All other event / style / encoding / error / node variants | unchanged (`pub const`) in value position; rename to SCREAMING_SNAKE_CASE in patterns | `unsafe_libyaml::YAML_*`                      |

For `yaml-rust2` and `noyalib`, see Path C and Path D above —
those crates do not aim for libyaml surface compatibility.

---

## Removed in 0.0.6

The deep internal modules that previous versions exposed leaked
implementation details of the hand-translated C copy. They are
**removed** in the shim. If your code depended on any of these,
the right replacement depends on which destination you chose:

| Removed from `libyml`                  | What it was                                          | Where it goes                                                                       |
| -------------------------------------- | ---------------------------------------------------- | ----------------------------------------------------------------------------------- |
| `libyml::api`                          | High-level wrappers over parser/emitter init pairs   | Bare functions at the `libyml` crate root (re-exported from `unsafe_libyaml`)       |
| `libyml::dumper`                       | `yaml_emitter_open` / `_close` / `_dump`             | `libyml::yaml_emitter_open` / `_close` / `_dump` (re-exported)                      |
| `libyml::decode`                       | `yaml_parser_initialize` / `_delete`                 | `libyml::yaml_parser_initialize` / `_delete` (re-exported)                          |
| `libyml::document`                     | `yaml_document_*` helpers                            | Some re-exported at the crate root; rest reachable via `unsafe_libyaml::*`          |
| `libyml::loader`                       | `yaml_parser_load`                                   | `libyml::yaml_parser_load` (re-exported)                                            |
| `libyml::memory`                       | `yaml_malloc` / `yaml_free` / `yaml_realloc` / `yaml_strdup` | None — the upstream uses Rust's `alloc` directly. Use Rust's standard primitives    |
| `libyml::string`                       | `yaml_string_extend` / `_join` helpers               | None — internal helpers of the C copy. Rewrite using Rust's `Vec` / `String`        |
| `libyml::yaml`                         | Public path-form re-exports of every enum + struct   | Type aliases at the `libyml` crate root; upstream snake_case names at `unsafe_libyaml::*` |
| `libyml::internal`                     | Hand-translated internal helpers                     | None — no longer reachable; the upstream is the source of truth                     |
| `libyml::macros`                       | Internal `do_loop!` / `__assert!` macros             | None — implementation details of the C copy                                         |
| `libyml::ops`                          | `ForceAdd` / `ForceInto` / `die` helpers             | None — the upstream uses its own internal equivalents                               |
| `libyml::utils`                        | Internal `memory_macros` module                      | None — implementation details of the C copy                                         |
| `libyml::success::Success` (nameable)  | `#[derive(PartialEq, Debug)]` struct wrapping `bool` | Read `.ok` on the upstream return value directly; the shim keeps `is_success(bool)` / `is_failure(bool)` |
| `libyml::run-emitter-test-suite` bin   | yaml-test-suite emitter runner                       | Upstream `unsafe-libyaml`'s own test suite covers the equivalent                    |
| `libyml::run-parser-test-suite` bin    | yaml-test-suite parser runner                        | Upstream `unsafe-libyaml`'s own test suite covers the equivalent                    |

This repository is archived — direct migration questions to the
destination crate's issue tracker.

---

## Behavioural notes

The shim is backed by `unsafe-libyaml`, whose upstream code has
diverged from the fork's snapshot in three user-visible ways:

1. **Boolean parameters take `bool`, not `c_int`.** Previously
   `yaml_scalar_event_initialize(..., 1, 1, style)` compiled with
   `c_int` arguments — `1` and `0` were valid values. Under the
   shim the function signature comes from `unsafe-libyaml`, so
   the same call site needs `true` / `false` instead. This is a
   hard compile error, not a silent change: the compiler points
   at every offending argument.

   ```diff
   -yaml_scalar_event_initialize(ev, anchor, tag, val, len, 1, 1, style);
   +yaml_scalar_event_initialize(ev, anchor, tag, val, len, true, true, style);
   ```

2. **`Success` is no longer a nameable type.** The upstream keeps
   its `Success` struct in a private module. The value still
   flows out of every `yaml_*` call and `.ok: bool` is still
   public, so reading the success flag works exactly as before —
   but you can no longer write a function signature mentioning
   the type:

   ```diff
   -fn check(r: libyml::success::Success) -> bool { is_success(r) }
   +fn check(ok: bool) -> bool { is_success(ok) }
   ```

   The retained `libyml::success::{is_success, is_failure}`
   helpers now take `bool` directly. Chain them as
   `is_success(call(...).ok)`.

3. **Enum variants rename PascalCase → SCREAMING_SNAKE_CASE in
   `match` arms.** The shim defines `pub const YamlUtf8Encoding`
   (etc.) so the historical names still work in **value
   position**:

   ```rust
   yaml_emitter_set_encoding(&mut emitter, YamlUtf8Encoding); // ok
   ```

   In refutable **patterns**, the upstream's SCREAMING_SNAKE_CASE
   name is required:

   ```diff
   - match enc { YamlUtf8Encoding => /* … */, _ => /* … */ }
   + match enc { unsafe_libyaml::YAML_UTF8_ENCODING => /* … */, _ => /* … */ }
   ```

   Both spellings are re-exported from `libyml`, so the imports
   side stays clean. The constraint is purely about how Rust
   resolves pattern arms vs. expressions.

Migrations to `yaml-rust2` or `noyalib` sidestep all three of
these because their public APIs don't share shapes with C
`libyaml` to begin with.

---

## MSRV

`libyml 0.0.6` requires **Rust 1.56.0** — the same floor as
`unsafe-libyaml`. The previous releases also required 1.56, so
this is not a bump.

---

## Test and example coverage in 0.0.6

The 0.0.6 shim is wire-compatible with typical user code (parser
/ emitter init + parse / emit cycles work transparently). The
original `libyml ≤ 0.0.5` test and example files are kept in this
repo where they could be brought across with a small mechanical
patch — they serve as a *worked-example* of what the migration
looks like from the downstream side. Where the original tests
probed the previous implementation's **private fields**, **derived
`Default` impls**, or **deleted internal modules**
(`internal`, `macros`, `externs`, the `string::yaml_string_extend`
unsound helper), they were removed because they reflect
implementation-detail coverage rather than downstream user code.

### Tests retained (3 files, 18 tests, all pass)

| File | Source | Changes applied | Tests |
| :--- | :--- | :--- | ---: |
| `tests/test_decode.rs` | from 0.0.5 | **verbatim** — `libyml::decode::*` path module re-exports through the shim | 8 |
| `tests/test_lib.rs` | from 0.0.5 | two-line patch (`is_success(call)` → `is_success(call.ok)`, drop `#![no_std]`) | 5 |
| `tests/shim.rs` | new | smoke suite covering parser init, parse-first-event, emit-mapping round-trip, type aliases, `success` helpers | 5 |

### Examples retained (3 runnable, all execute to completion)

| Path | Source | Changes applied |
| :--- | :--- | :--- |
| `examples/example.rs` | from 0.0.5 (aggregator shape) | runs `examples/apis/main.rs` then a parse + emit demo |
| `examples/apis/main.rs` | from 0.0.5 | parser slabs kept; `memory` + `string` slabs kept as commented-out blocks with Rust-native replacements inline |
| `examples/migration.rs` | new | single-file shim demo (parse a 2-line doc and count events) |

### Tests removed (probed the old implementation's private shape)

| File | Why |
| :--- | :--- |
| `tests/test_api.rs` | Used `libyml::memory::yaml_malloc` / `_strdup` and `libyml::externs::free` — the C-libyaml allocator surface is removed in 0.0.6 (the upstream uses Rust's `alloc` directly) |
| `tests/test_document.rs` | Called `YamlDocumentT::cleanup()` and constructed `yaml_mark_t::default()` — internal helpers not exposed by `unsafe-libyaml` |
| `tests/test_dumper.rs` | Read private fields (`emitter.opened`, `emitter.closed`, `emitter.write_handler`) — public in the fork's reimplementation, private in `unsafe-libyaml` |
| `tests/test_emitter.rs` | Imported the deleted `src/bin/run-emitter-test-suite.rs` runner |
| `tests/test_internal.rs` | Tested the removed `libyml::internal` module |
| `tests/test_loader.rs` | Imported `libyml::loader::yaml_parser_set_composer_error` — internal helper not in the upstream's public API |
| `tests/test_macros.rs` | Imported the removed `libyml::macros`, `libyml::libc`, `libyml::externs` and the `yaml_string_extend` unsound helper (RUSTSEC-2025-0067) |
| `tests/test_memory.rs` | Imported the removed `libyml::memory` allocator wrappers |
| `tests/test_parser.rs`, `test_parser_error.rs` | Imported the deleted `src/bin/run-parser-test-suite.rs` runner |
| `tests/test_string.rs` | Imported the removed `libyml::string` module (the unsound `yaml_string_extend` helper RUSTSEC-2025-0067 covers) |
| `tests/test_yaml.rs` | Called `YamlEncodingT::default()` and used `YamlAnyEncoding` / `YamlAnyScalarStyle` / etc. as enum-variant `match` patterns — both rely on derive impls and the variant-position rename gap |
| `tests/data/*` (libyml-test-suite proc macros) | yaml-test-suite harness; upstream `unsafe-libyaml` runs its own equivalent suite |

### Examples removed

| Path | Why |
| :--- | :--- |
| `src/bin/run-emitter-test-suite.rs` | Test-suite runner depending on removed internals |
| `src/bin/run-parser-test-suite.rs` | Test-suite runner depending on removed internals |
| `src/bin/cstr/*` | Internal CStr helper for the removed test-suite binaries |

If you depended on any of these, pick the destination crate from
the table at the top of this document — its public surface
offers the equivalent functionality.
