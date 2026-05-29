// SPDX-License-Identifier: MIT OR Apache-2.0

//! # ⚠️ `libyml` is deprecated — migrate to a maintained alternative
//!
//! This crate is **unmaintained**. The `0.0.6` release is a thin
//! compatibility shim so existing call sites keep working while you
//! plan a migration. See [`MIGRATION.md`](https://github.com/sebastienrousseau/libyml/blob/master/MIGRATION.md)
//! for the full guide.
//!
//! ## Maintained alternatives
//!
//! - **[`unsafe-libyaml`](https://crates.io/crates/unsafe-libyaml)**
//!   — the upstream Rust translation of C `libyaml` that `libyml`
//!   was originally forked from. Same `yaml_*` function surface,
//!   actively maintained. **Drop-in replacement** for users on the
//!   raw FFI-shaped API.
//! - **[`yaml-rust2`](https://crates.io/crates/yaml-rust2)** —
//!   pure-Rust low-level parser, no FFI. Returns a `Yaml` enum AST
//!   instead of the event-stream model. Fits users who want to move
//!   off the C-libyaml shape entirely while keeping a low-level
//!   parser primitive.
//! - **[`noyalib`](https://crates.io/crates/noyalib)** — modern,
//!   pure-Rust, `#![forbid(unsafe_code)]` YAML library with a
//!   high-level typed API (`from_str::<T>` / `Value`). Fits users
//!   who can move from event-stream parsing to typed deserialisation.
//!
//! `MIGRATION.md` carries the per-crate mapping tables.
//!
//! ## Why the shim is backed by `unsafe-libyaml`
//!
//! `libyml` was originally a fork of `unsafe-libyaml` with cosmetic
//! renames (snake_case → PascalCase for type names). The 0.0.6 shim
//! reverts those renames internally and re-exports the upstream's
//! functions and types, restoring the historical PascalCase aliases
//! so existing call sites compile unchanged.
//!
//! This is an implementation detail, not a recommendation that you
//! must use `unsafe-libyaml`. Two things follow:
//!
//! - **No duplicated `unsafe` translation in the dependency graph.**
//!   Downstream users link the upstream's audited copy of the
//!   C-libyaml translation rather than this fork's stale copy.
//! - **Bug fixes flow through.** Anything fixed in
//!   `unsafe-libyaml` lands in users of this shim on a plain
//!   `cargo update`, without a new `libyml` release.
//!
//! If you want to evaluate `yaml-rust2` or `noyalib` directly,
//! `MIGRATION.md` covers both.
//!
//! ## Stop-gap: keep using `libyml = "0.0.6"`
//!
//! Existing call sites compile unchanged against this shim. Every
//! item below is marked `#[deprecated]`, so the compiler will point
//! at the spots that need updating during your migration.
//!
//! ## Removed in 0.0.6 (vs. 0.0.5)
//!
//! The deep internal modules that previous versions exposed —
//! `libyml::api`, `libyml::dumper`, `libyml::decode`,
//! `libyml::document`, `libyml::loader`, `libyml::memory`,
//! `libyml::string`, `libyml::success`, the public `yaml`
//! module — are **gone** in this release. Their hand-translated
//! C implementations have been replaced by re-exports of the
//! upstream's equivalents. See `MIGRATION.md` for the equivalence
//! table per alternative.

#![deprecated(
    since = "0.0.6",
    note = "libyml is unmaintained. Migrate to a maintained alternative (unsafe-libyaml, yaml-rust2, or noyalib). See MIGRATION.md."
)]
#![doc(html_root_url = "https://docs.rs/libyml/0.0.6")]
#![no_std]
// The PascalCase `pub const Yaml*` aliases below intentionally
// shadow the upstream's SCREAMING_SNAKE_CASE variants so existing
// `libyml`-flavoured call sites compile unchanged. The
// `non_upper_case_globals` lint flags this naming convention as
// non-idiomatic; the alias is the entire point.
#![allow(non_upper_case_globals)]

// ── Top-level function re-exports — name-for-name with libyml 0.0.5 ────

#[doc(inline)]
pub use unsafe_libyaml::{
    yaml_alias_event_initialize, yaml_document_delete,
    yaml_document_end_event_initialize, yaml_document_get_node,
    yaml_document_get_root_node, yaml_document_initialize,
    yaml_document_start_event_initialize, yaml_emitter_close,
    yaml_emitter_delete, yaml_emitter_dump, yaml_emitter_emit,
    yaml_emitter_flush, yaml_emitter_initialize, yaml_emitter_open,
    yaml_emitter_set_break, yaml_emitter_set_canonical,
    yaml_emitter_set_encoding, yaml_emitter_set_indent,
    yaml_emitter_set_output, yaml_emitter_set_output_string,
    yaml_emitter_set_unicode, yaml_emitter_set_width,
    yaml_event_delete, yaml_mapping_end_event_initialize,
    yaml_mapping_start_event_initialize, yaml_parser_delete,
    yaml_parser_initialize, yaml_parser_load, yaml_parser_parse,
    yaml_parser_scan, yaml_parser_set_encoding, yaml_parser_set_input,
    yaml_parser_set_input_string, yaml_scalar_event_initialize,
    yaml_sequence_end_event_initialize,
    yaml_sequence_start_event_initialize,
    yaml_stream_end_event_initialize,
    yaml_stream_start_event_initialize, yaml_token_delete,
};

// ── Type aliases — restore the PascalCase names libyml ≤ 0.0.5 used ────
//
// `unsafe-libyaml` uses C-style snake_case (`yaml_event_t`,
// `yaml_parser_t`, …); `libyml` historically renamed those to
// PascalCase (`YamlEventT`, `YamlParserT`, …). The aliases below
// preserve the historical name surface so `use libyml::YamlParserT`
// keeps resolving.

/// Alias for [`unsafe_libyaml::yaml_alias_data_t`].
pub type YamlAliasDataT = unsafe_libyaml::yaml_alias_data_t;
/// Alias for [`unsafe_libyaml::yaml_break_t`].
pub type YamlBreakT = unsafe_libyaml::yaml_break_t;
/// Alias for [`unsafe_libyaml::yaml_document_t`].
pub type YamlDocumentT = unsafe_libyaml::yaml_document_t;
/// Alias for [`unsafe_libyaml::yaml_emitter_state_t`].
pub type YamlEmitterStateT = unsafe_libyaml::yaml_emitter_state_t;
/// Alias for [`unsafe_libyaml::yaml_emitter_t`].
pub type YamlEmitterT = unsafe_libyaml::yaml_emitter_t;
/// Alias for [`unsafe_libyaml::yaml_encoding_t`].
pub type YamlEncodingT = unsafe_libyaml::yaml_encoding_t;
/// Alias for [`unsafe_libyaml::yaml_error_type_t`].
pub type YamlErrorTypeT = unsafe_libyaml::yaml_error_type_t;
/// Alias for [`unsafe_libyaml::yaml_event_t`].
pub type YamlEventT = unsafe_libyaml::yaml_event_t;
/// Alias for [`unsafe_libyaml::yaml_event_type_t`].
pub type YamlEventTypeT = unsafe_libyaml::yaml_event_type_t;
/// Alias for [`unsafe_libyaml::yaml_mapping_style_t`].
pub type YamlMappingStyleT = unsafe_libyaml::yaml_mapping_style_t;
/// Alias for [`unsafe_libyaml::yaml_mark_t`].
pub type YamlMarkT = unsafe_libyaml::yaml_mark_t;
/// Alias for [`unsafe_libyaml::yaml_node_item_t`].
pub type YamlNodeItemT = unsafe_libyaml::yaml_node_item_t;
/// Alias for [`unsafe_libyaml::yaml_node_pair_t`].
pub type YamlNodePairT = unsafe_libyaml::yaml_node_pair_t;
/// Alias for [`unsafe_libyaml::yaml_node_t`].
pub type YamlNodeT = unsafe_libyaml::yaml_node_t;
/// Alias for [`unsafe_libyaml::yaml_node_type_t`].
pub type YamlNodeTypeT = unsafe_libyaml::yaml_node_type_t;
/// Alias for [`unsafe_libyaml::yaml_parser_state_t`].
pub type YamlParserStateT = unsafe_libyaml::yaml_parser_state_t;
/// Alias for [`unsafe_libyaml::yaml_parser_t`].
pub type YamlParserT = unsafe_libyaml::yaml_parser_t;
/// Alias for [`unsafe_libyaml::yaml_read_handler_t`].
pub type YamlReadHandlerT = unsafe_libyaml::yaml_read_handler_t;
/// Alias for [`unsafe_libyaml::yaml_scalar_style_t`].
pub type YamlScalarStyleT = unsafe_libyaml::yaml_scalar_style_t;
/// Alias for [`unsafe_libyaml::yaml_sequence_style_t`].
pub type YamlSequenceStyleT = unsafe_libyaml::yaml_sequence_style_t;
/// Alias for [`unsafe_libyaml::yaml_simple_key_t`].
pub type YamlSimpleKeyT = unsafe_libyaml::yaml_simple_key_t;
/// Alias for [`unsafe_libyaml::yaml_stack_t`].
pub type YamlStackT<T> = unsafe_libyaml::yaml_stack_t<T>;
/// Alias for [`unsafe_libyaml::yaml_tag_directive_t`].
pub type YamlTagDirectiveT = unsafe_libyaml::yaml_tag_directive_t;
/// Alias for [`unsafe_libyaml::yaml_token_t`].
pub type YamlTokenT = unsafe_libyaml::yaml_token_t;
/// Alias for [`unsafe_libyaml::yaml_token_type_t`].
pub type YamlTokenTypeT = unsafe_libyaml::yaml_token_type_t;
/// Alias for [`unsafe_libyaml::yaml_version_directive_t`].
pub type YamlVersionDirectiveT =
    unsafe_libyaml::yaml_version_directive_t;
/// Alias for [`unsafe_libyaml::yaml_write_handler_t`].
pub type YamlWriteHandlerT = unsafe_libyaml::yaml_write_handler_t;

// ── Enum-variant re-exports ───────────────────────────────────────────
//
// `libyml` ≤ 0.0.5 named its enum variants in PascalCase
// (`YamlUtf8Encoding`, `YamlPlainScalarStyle`, …) and re-exported
// them at the crate root via `pub use crate::yaml::*::*`.
// `unsafe-libyaml` keeps the C convention — SCREAMING_SNAKE_CASE
// (`YAML_UTF8_ENCODING`, `YAML_PLAIN_SCALAR_STYLE`, …).
//
// To preserve the historical bare-name surface, this section
// declares `pub const`s that alias each PascalCase name to the
// equivalent upstream variant. The aliases work in **value
// position** (`yaml_emitter_set_encoding(emit, YamlUtf8Encoding)`)
// — which is the overwhelming majority of usages — but **not as
// refutable patterns** in `match` arms, where the upstream
// SCREAMING_SNAKE_CASE name is required. The MIGRATION.md guide
// documents this delta.
//
// The deep parser/emitter state-machine enums (`YamlParserStateT`,
// `YamlEmitterStateT`) had ~30 variants each and were never part
// of typical user code; their variants are reachable through the
// upstream's snake_case path (`unsafe_libyaml::YAML_PARSE_*`).

/// Re-export of upstream variants under their original SCREAMING_SNAKE_CASE
/// names. Available for users who want to opt into the upstream surface
/// without renaming.
#[doc(hidden)]
pub use unsafe_libyaml::{
    YAML_ALIAS_EVENT, YAML_ALIAS_TOKEN, YAML_ANCHOR_TOKEN,
    YAML_ANY_ENCODING, YAML_ANY_MAPPING_STYLE, YAML_ANY_SCALAR_STYLE,
    YAML_ANY_SEQUENCE_STYLE, YAML_BLOCK_END_TOKEN,
    YAML_BLOCK_ENTRY_TOKEN, YAML_BLOCK_MAPPING_START_TOKEN,
    YAML_BLOCK_MAPPING_STYLE, YAML_BLOCK_SEQUENCE_START_TOKEN,
    YAML_BLOCK_SEQUENCE_STYLE, YAML_COMPOSER_ERROR,
    YAML_DOCUMENT_END_EVENT, YAML_DOCUMENT_END_TOKEN,
    YAML_DOCUMENT_START_EVENT, YAML_DOCUMENT_START_TOKEN,
    YAML_DOUBLE_QUOTED_SCALAR_STYLE, YAML_EMITTER_ERROR,
    YAML_FLOW_ENTRY_TOKEN, YAML_FLOW_MAPPING_END_TOKEN,
    YAML_FLOW_MAPPING_START_TOKEN, YAML_FLOW_MAPPING_STYLE,
    YAML_FLOW_SEQUENCE_END_TOKEN, YAML_FLOW_SEQUENCE_START_TOKEN,
    YAML_FLOW_SEQUENCE_STYLE, YAML_FOLDED_SCALAR_STYLE, YAML_KEY_TOKEN,
    YAML_LITERAL_SCALAR_STYLE, YAML_MAPPING_END_EVENT,
    YAML_MAPPING_NODE, YAML_MAPPING_START_EVENT, YAML_MEMORY_ERROR,
    YAML_NO_ERROR, YAML_NO_EVENT, YAML_NO_NODE, YAML_NO_TOKEN,
    YAML_PARSER_ERROR, YAML_PLAIN_SCALAR_STYLE, YAML_READER_ERROR,
    YAML_SCALAR_EVENT, YAML_SCALAR_NODE, YAML_SCALAR_TOKEN,
    YAML_SCANNER_ERROR, YAML_SEQUENCE_END_EVENT, YAML_SEQUENCE_NODE,
    YAML_SEQUENCE_START_EVENT, YAML_SINGLE_QUOTED_SCALAR_STYLE,
    YAML_STREAM_END_EVENT, YAML_STREAM_END_TOKEN,
    YAML_STREAM_START_EVENT, YAML_STREAM_START_TOKEN,
    YAML_TAG_DIRECTIVE_TOKEN, YAML_TAG_TOKEN, YAML_UTF16BE_ENCODING,
    YAML_UTF16LE_ENCODING, YAML_UTF8_ENCODING, YAML_VALUE_TOKEN,
    YAML_VERSION_DIRECTIVE_TOKEN, YAML_WRITER_ERROR,
};

// ── PascalCase const aliases for libyml ≤ 0.0.5 callers ───────────────

/// Alias for [`unsafe_libyaml::YAML_ANY_SCALAR_STYLE`].
pub const YamlAnyScalarStyle: YamlScalarStyleT =
    unsafe_libyaml::YAML_ANY_SCALAR_STYLE;
/// Alias for [`unsafe_libyaml::YAML_PLAIN_SCALAR_STYLE`].
pub const YamlPlainScalarStyle: YamlScalarStyleT =
    unsafe_libyaml::YAML_PLAIN_SCALAR_STYLE;
/// Alias for [`unsafe_libyaml::YAML_SINGLE_QUOTED_SCALAR_STYLE`].
pub const YamlSingleQuotedScalarStyle: YamlScalarStyleT =
    unsafe_libyaml::YAML_SINGLE_QUOTED_SCALAR_STYLE;
/// Alias for [`unsafe_libyaml::YAML_DOUBLE_QUOTED_SCALAR_STYLE`].
pub const YamlDoubleQuotedScalarStyle: YamlScalarStyleT =
    unsafe_libyaml::YAML_DOUBLE_QUOTED_SCALAR_STYLE;
/// Alias for [`unsafe_libyaml::YAML_LITERAL_SCALAR_STYLE`].
pub const YamlLiteralScalarStyle: YamlScalarStyleT =
    unsafe_libyaml::YAML_LITERAL_SCALAR_STYLE;
/// Alias for [`unsafe_libyaml::YAML_FOLDED_SCALAR_STYLE`].
pub const YamlFoldedScalarStyle: YamlScalarStyleT =
    unsafe_libyaml::YAML_FOLDED_SCALAR_STYLE;

/// Alias for [`unsafe_libyaml::YAML_ANY_SEQUENCE_STYLE`].
pub const YamlAnySequenceStyle: YamlSequenceStyleT =
    unsafe_libyaml::YAML_ANY_SEQUENCE_STYLE;
/// Alias for [`unsafe_libyaml::YAML_BLOCK_SEQUENCE_STYLE`].
pub const YamlBlockSequenceStyle: YamlSequenceStyleT =
    unsafe_libyaml::YAML_BLOCK_SEQUENCE_STYLE;
/// Alias for [`unsafe_libyaml::YAML_FLOW_SEQUENCE_STYLE`].
pub const YamlFlowSequenceStyle: YamlSequenceStyleT =
    unsafe_libyaml::YAML_FLOW_SEQUENCE_STYLE;

/// Alias for [`unsafe_libyaml::YAML_ANY_MAPPING_STYLE`].
pub const YamlAnyMappingStyle: YamlMappingStyleT =
    unsafe_libyaml::YAML_ANY_MAPPING_STYLE;
/// Alias for [`unsafe_libyaml::YAML_BLOCK_MAPPING_STYLE`].
pub const YamlBlockMappingStyle: YamlMappingStyleT =
    unsafe_libyaml::YAML_BLOCK_MAPPING_STYLE;
/// Alias for [`unsafe_libyaml::YAML_FLOW_MAPPING_STYLE`].
pub const YamlFlowMappingStyle: YamlMappingStyleT =
    unsafe_libyaml::YAML_FLOW_MAPPING_STYLE;

/// Alias for [`unsafe_libyaml::YAML_ANY_ENCODING`].
pub const YamlAnyEncoding: YamlEncodingT =
    unsafe_libyaml::YAML_ANY_ENCODING;
/// Alias for [`unsafe_libyaml::YAML_UTF8_ENCODING`].
pub const YamlUtf8Encoding: YamlEncodingT =
    unsafe_libyaml::YAML_UTF8_ENCODING;
/// Alias for [`unsafe_libyaml::YAML_UTF16LE_ENCODING`].
pub const YamlUtf16leEncoding: YamlEncodingT =
    unsafe_libyaml::YAML_UTF16LE_ENCODING;
/// Alias for [`unsafe_libyaml::YAML_UTF16BE_ENCODING`].
pub const YamlUtf16beEncoding: YamlEncodingT =
    unsafe_libyaml::YAML_UTF16BE_ENCODING;

/// Alias for [`unsafe_libyaml::YAML_NO_ERROR`].
pub const YamlNoError: YamlErrorTypeT = unsafe_libyaml::YAML_NO_ERROR;
/// Alias for [`unsafe_libyaml::YAML_MEMORY_ERROR`].
pub const YamlMemoryError: YamlErrorTypeT =
    unsafe_libyaml::YAML_MEMORY_ERROR;
/// Alias for [`unsafe_libyaml::YAML_READER_ERROR`].
pub const YamlReaderError: YamlErrorTypeT =
    unsafe_libyaml::YAML_READER_ERROR;
/// Alias for [`unsafe_libyaml::YAML_SCANNER_ERROR`].
pub const YamlScannerError: YamlErrorTypeT =
    unsafe_libyaml::YAML_SCANNER_ERROR;
/// Alias for [`unsafe_libyaml::YAML_PARSER_ERROR`].
pub const YamlParserError: YamlErrorTypeT =
    unsafe_libyaml::YAML_PARSER_ERROR;
/// Alias for [`unsafe_libyaml::YAML_COMPOSER_ERROR`].
pub const YamlComposerError: YamlErrorTypeT =
    unsafe_libyaml::YAML_COMPOSER_ERROR;
/// Alias for [`unsafe_libyaml::YAML_WRITER_ERROR`].
pub const YamlWriterError: YamlErrorTypeT =
    unsafe_libyaml::YAML_WRITER_ERROR;
/// Alias for [`unsafe_libyaml::YAML_EMITTER_ERROR`].
pub const YamlEmitterError: YamlErrorTypeT =
    unsafe_libyaml::YAML_EMITTER_ERROR;

/// Alias for [`unsafe_libyaml::YAML_NO_EVENT`].
pub const YamlNoEvent: YamlEventTypeT = unsafe_libyaml::YAML_NO_EVENT;
/// Alias for [`unsafe_libyaml::YAML_STREAM_START_EVENT`].
pub const YamlStreamStartEvent: YamlEventTypeT =
    unsafe_libyaml::YAML_STREAM_START_EVENT;
/// Alias for [`unsafe_libyaml::YAML_STREAM_END_EVENT`].
pub const YamlStreamEndEvent: YamlEventTypeT =
    unsafe_libyaml::YAML_STREAM_END_EVENT;
/// Alias for [`unsafe_libyaml::YAML_DOCUMENT_START_EVENT`].
pub const YamlDocumentStartEvent: YamlEventTypeT =
    unsafe_libyaml::YAML_DOCUMENT_START_EVENT;
/// Alias for [`unsafe_libyaml::YAML_DOCUMENT_END_EVENT`].
pub const YamlDocumentEndEvent: YamlEventTypeT =
    unsafe_libyaml::YAML_DOCUMENT_END_EVENT;
/// Alias for [`unsafe_libyaml::YAML_ALIAS_EVENT`].
pub const YamlAliasEvent: YamlEventTypeT =
    unsafe_libyaml::YAML_ALIAS_EVENT;
/// Alias for [`unsafe_libyaml::YAML_SCALAR_EVENT`].
pub const YamlScalarEvent: YamlEventTypeT =
    unsafe_libyaml::YAML_SCALAR_EVENT;
/// Alias for [`unsafe_libyaml::YAML_SEQUENCE_START_EVENT`].
pub const YamlSequenceStartEvent: YamlEventTypeT =
    unsafe_libyaml::YAML_SEQUENCE_START_EVENT;
/// Alias for [`unsafe_libyaml::YAML_SEQUENCE_END_EVENT`].
pub const YamlSequenceEndEvent: YamlEventTypeT =
    unsafe_libyaml::YAML_SEQUENCE_END_EVENT;
/// Alias for [`unsafe_libyaml::YAML_MAPPING_START_EVENT`].
pub const YamlMappingStartEvent: YamlEventTypeT =
    unsafe_libyaml::YAML_MAPPING_START_EVENT;
/// Alias for [`unsafe_libyaml::YAML_MAPPING_END_EVENT`].
pub const YamlMappingEndEvent: YamlEventTypeT =
    unsafe_libyaml::YAML_MAPPING_END_EVENT;

/// Alias for [`unsafe_libyaml::YAML_NO_NODE`].
pub const YamlNoNode: YamlNodeTypeT = unsafe_libyaml::YAML_NO_NODE;
/// Alias for [`unsafe_libyaml::YAML_SCALAR_NODE`].
pub const YamlScalarNode: YamlNodeTypeT =
    unsafe_libyaml::YAML_SCALAR_NODE;
/// Alias for [`unsafe_libyaml::YAML_SEQUENCE_NODE`].
pub const YamlSequenceNode: YamlNodeTypeT =
    unsafe_libyaml::YAML_SEQUENCE_NODE;
/// Alias for [`unsafe_libyaml::YAML_MAPPING_NODE`].
pub const YamlMappingNode: YamlNodeTypeT =
    unsafe_libyaml::YAML_MAPPING_NODE;

// ── `libyml::success` — keep path-form imports working ────────────────

/// Success/failure helpers retained for source compatibility with
/// `libyml ≤ 0.0.5`.
///
/// **Migration note.** The upstream `unsafe-libyaml` crate keeps its
/// `Success` / `Failure` structs in a private module — the values
/// flow out of `yaml_*` calls but cannot be named at a path. The
/// previous `libyml::success::Success` type therefore has **no
/// nameable equivalent in the shim**; the public helpers below
/// accept `bool` so they can still chain with the upstream return
/// values via `.ok`.
///
/// Old:
///
/// ```ignore
/// use libyml::success::is_success;
/// if is_success(yaml_parser_initialize(p)) { /* … */ }
/// ```
///
/// New:
///
/// ```ignore
/// use libyml::success::is_success;
/// if is_success(yaml_parser_initialize(p).ok) { /* … */ }
/// ```
pub mod success {
    /// Returns `true` when the operation was successful.
    ///
    /// Historical `libyml ≤ 0.0.5` helper. Now takes the `ok`
    /// flag of the upstream `Success` struct directly.
    pub fn is_success(ok: bool) -> bool {
        ok
    }

    /// Returns `true` when the operation failed.
    ///
    /// Historical `libyml ≤ 0.0.5` helper. Now takes the `ok`
    /// flag of the upstream `Success` struct directly.
    pub fn is_failure(ok: bool) -> bool {
        !ok
    }
}
