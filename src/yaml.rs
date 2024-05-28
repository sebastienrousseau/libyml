
use crate::libc;
use core::ops::Deref;
use core::ptr::{self, addr_of};

pub(crate) use self::{
    YamlEncodingT::*, YamlEventTypeT::*, YamlNodeTypeT::*,
};
pub(crate) use core::primitive::{
    i64 as ptrdiff_t, u64 as size_t, u8 as yaml_char_t,
};

/// The version directive data.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlVersionDirectiveT {
    /// The major version number.
    pub major: libc::c_int,
    /// The minor version number.
    pub minor: libc::c_int,
}

/// The tag directive data.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlTagDirectiveT {
    /// The tag handle.
    pub handle: *mut yaml_char_t,
    /// The tag prefix.
    pub prefix: *mut yaml_char_t,
}

/// The stream encoding.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
#[non_exhaustive]
pub enum YamlEncodingT {
    /// Let the parser choose the encoding.
    YamlAnyEncoding = 0,
    /// The default UTF-8 encoding.
    YamlUtf8Encoding = 1,
    /// The UTF-16-LE encoding with BOM.
    YamlUtf16leEncoding = 2,
    /// The UTF-16-BE encoding with BOM.
    YamlUtf16beEncoding = 3,
}

/// Line break type.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
#[non_exhaustive]
pub enum YamlBreakT {
    /// Let the parser choose the break type.
    YamlAnyBreak = 0,
    /// Use CR for line breaks (Mac style).
    YamlCrBreak = 1,
    /// Use LN for line breaks (Unix style).
    YamlLnBreak = 2,
    /// Use CR LN for line breaks (DOS style).
    YamlCrlnBreak = 3,
}

/// Many bad things could happen with the parser and emitter.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
#[non_exhaustive]
pub enum YamlErrorTypeT {
    /// No error is produced.
    YamlNoError = 0,
    /// Cannot allocate or reallocate a block of memory.
    YamlMemoryError = 1,
    /// Cannot read or decode the input stream.
    YamlReaderError = 2,
    /// Cannot scan the input stream.
    YamlScannerError = 3,
    /// Cannot parse the input stream.
    YamlParserError = 4,
    /// Cannot compose a YAML document.
    YamlComposerError = 5,
    /// Cannot write to the output stream.
    YamlWriterError = 6,
    /// Cannot emit a YAML stream.
    YamlEmitterError = 7,
}

/// The pointer position.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlMarkT {
    /// The position index.
    pub index: size_t,
    /// The position line.
    pub line: size_t,
    /// The position column.
    pub column: size_t,
}

/// Scalar styles.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
#[non_exhaustive]
pub enum YamlScalarStyleT {
    /// Let the emitter choose the style.
    YamlAnyScalarStyle = 0,
    /// The plain scalar style.
    YamlPlainScalarStyle = 1,
    /// The single-quoted scalar style.
    YamlSingleQuotedScalarStyle = 2,
    /// The double-quoted scalar style.
    YamlDoubleQuotedScalarStyle = 3,
    /// The literal scalar style.
    YamlLiteralScalarStyle = 4,
    /// The folded scalar style.
    YamlFoldedScalarStyle = 5,
}

/// Sequence styles.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
#[non_exhaustive]
pub enum YamlSequenceStyleT {
    /// Let the emitter choose the style.
    YamlAnySequenceStyle = 0,
    /// The block sequence style.
    YamlBlockSequenceStyle = 1,
    /// The flow sequence style.
    YamlFlowSequenceStyle = 2,
}

/// Mapping styles.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
#[non_exhaustive]
pub enum YamlMappingStyleT {
    /// Let the emitter choose the style.
    YamlAnyMappingStyle = 0,
    /// The block mapping style.
    YamlBlockMappingStyle = 1,
    /// The flow mapping style.
    YamlFlowMappingStyle = 2,
}

/// Token types.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
#[non_exhaustive]
pub enum YamlTokenTypeT {
    /// An empty token.
    YamlNoToken = 0,
    /// A stream-start token.
    YamlStreamStartToken = 1,
    /// A stream-end token.
    YamlStreamEndToken = 2,
    /// A version-directive token.
    YamlVersionDirectiveToken = 3,
    /// A tag-directive token.
    YamlTagDirectiveToken = 4,
    /// A document-start token.
    YamlDocumentStartToken = 5,
    /// A document-end token.
    YamlDocumentEndToken = 6,
    /// A block-sequence-start token.
    YamlBlockSequenceStartToken = 7,
    /// A block-mapping-start token.
    YamlBlockMappingStartToken = 8,
    /// A block-end token.
    YamlBlockEndToken = 9,
    /// A flow-sequence-start token.
    YamlFlowSequenceStartToken = 10,
    /// A flow-sequence-end token.
    YamlFlowSequenceEndToken = 11,
    /// A flow-mapping-start token.
    YamlFlowMappingStartToken = 12,
    /// A flow-mapping-end token.
    YamlFlowMappingEndToken = 13,
    /// A block-entry token.
    YamlBlockEntryToken = 14,
    /// A flow-entry token.
    YamlFlowEntryToken = 15,
    /// A key token.
    YamlKeyToken = 16,
    /// A value token.
    YamlValueToken = 17,
    /// An alias token.
    YamlAliasToken = 18,
    /// An anchor token.
    YamlAnchorToken = 19,
    /// A tag token.
    YamlTagToken = 20,
    /// A scalar token.
    YamlScalarToken = 21,
}

/// The token structure.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlTokenT {
    /// The token type.
    pub type_: YamlTokenTypeT,
    /// The token data.
    ///
    /// ```
    /// # const _: &str = stringify! {
    /// union {
    ///     /// The stream start (for YamlStreamStartToken).
    ///     stream_start: struct {
    ///         /// The stream encoding.
    ///         encoding: YamlEncodingT,
    ///     },
    ///     /// The alias (for YamlAliasToken).
    ///     alias: struct {
    ///         /// The alias value.
    ///         value: *mut u8,
    ///     },
    ///     /// The anchor (for YamlAnchorToken).
    ///     anchor: struct {
    ///         /// The anchor value.
    ///         value: *mut u8,
    ///     },
    ///     /// The tag (for YamlTagToken).
    ///     tag: struct {
    ///         /// The tag handle.
    ///         handle: *mut u8,
    ///         /// The tag suffix.
    ///         suffix: *mut u8,
    ///     },
    ///     /// The scalar value (for YamlScalarToken).
    ///     scalar: struct {
    ///         /// The scalar value.
    ///         value: *mut u8,
    ///         /// The length of the scalar value.
    ///         length: u64,
    ///         /// The scalar style.
    ///         style: YamlScalarStyleT,
    ///     },
    ///     /// The version directive (for YamlVersionDirectiveToken).
    ///     version_directive: struct {
    ///         /// The major version number.
    ///         major: i32,
    ///         /// The minor version number.
    ///         minor: i32,
    ///     },
    ///     /// The tag directive (for YamlTagDirectiveToken).
    ///     tag_directive: struct {
    ///         /// The tag handle.
    ///         handle: *mut u8,
    ///         /// The tag prefix.
    ///         prefix: *mut u8,
    ///     },
    /// }
    /// # };
    /// ```
    pub data: UnnamedYamlTokenTData,
    /// The beginning of the token.
    pub start_mark: YamlMarkT,
    /// The end of the token.
    pub end_mark: YamlMarkT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct UnnamedYamlTokenTData {
    /// The stream start (for YamlStreamStartToken).
    pub stream_start: UnnamedYamlTokenTdataStreamStart,
    /// The alias (for YamlAliasToken).
    pub alias: UnnamedYamlTokenTdataAlias,
    /// The anchor (for YamlAnchorToken).
    pub anchor: UnnamedYamlTokenTdataAnchor,
    /// The tag (for YamlTagToken).
    pub tag: UnnamedYamlTokenTdataTag,
    /// The scalar value (for YamlScalarToken).
    pub scalar: UnnamedYamlTokenTdataScalar,
    /// The version directive (for YamlVersionDirectiveToken).
    pub version_directive: UnnamedYamlTokenTdataVersionDirective,
    /// The tag directive (for YamlTagDirectiveToken).
    pub tag_directive: UnnamedYamlTokenTdataTagDirective,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlTokenTdataStreamStart {
    /// The stream encoding.
    pub encoding: YamlEncodingT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlTokenTdataAlias {
    /// The alias value.
    pub value: *mut yaml_char_t,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlTokenTdataAnchor {
    /// The anchor value.
    pub value: *mut yaml_char_t,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlTokenTdataTag {
    /// The tag handle.
    pub handle: *mut yaml_char_t,
    /// The tag suffix.
    pub suffix: *mut yaml_char_t,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlTokenTdataScalar {
    /// The scalar value.
    pub value: *mut yaml_char_t,
    /// The length of the scalar value.
    pub length: size_t,
    /// The scalar style.
    pub style: YamlScalarStyleT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlTokenTdataVersionDirective {
    /// The major version number.
    pub major: libc::c_int,
    /// The minor version number.
    pub minor: libc::c_int,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlTokenTdataTagDirective {
    /// The tag handle.
    pub handle: *mut yaml_char_t,
    /// The tag prefix.
    pub prefix: *mut yaml_char_t,
}

/// Event types.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
#[non_exhaustive]
pub enum YamlEventTypeT {
    /// An empty event.
    YamlNoEvent = 0,
    /// A stream-start event.
    YamlStreamStartEvent = 1,
    /// A stream-end event.
    YamlStreamEndEvent = 2,
    /// A document-start event.
    YamlDocumentStartEvent = 3,
    /// A document-end event.
    YamlDocumentEndEvent = 4,
    /// An alias event.
    YamlAliasEvent = 5,
    /// A scalar event.
    YamlScalarEvent = 6,
    /// A sequence-start event.
    YamlSequenceStartEvent = 7,
    /// A sequence-end event.
    YamlSequenceEndEvent = 8,
    /// A mapping-start event.
    YamlMappingStartEvent = 9,
    /// A mapping-end event.
    YamlMappingEndEvent = 10,
}

/// The event structure.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlEventT {
    /// The event type.
    pub type_: YamlEventTypeT,
    /// The event data.
    ///
    /// ```
    /// # const _: &str = stringify! {
    /// union {
    ///     /// The stream parameters (for YamlStreamStartEvent).
    ///     stream_start: struct {
    ///         /// The document encoding.
    ///         encoding: YamlEncodingT,
    ///     },
    ///     /// The document parameters (for YamlDocumentStartEvent).
    ///     document_start: struct {
    ///         /// The version directive.
    ///         version_directive: *mut YamlVersionDirectiveT,
    ///         /// The list of tag directives.
    ///         tag_directives: struct {
    ///             /// The beginning of the tag directives list.
    ///             start: *mut YamlTagDirectiveT,
    ///             /// The end of the tag directives list.
    ///             end: *mut YamlTagDirectiveT,
    ///         },
    ///         /// Is the document indicator implicit?
    ///         implicit: i32,
    ///     },
    ///     /// The document end parameters (for YamlDocumentEndEvent).
    ///     document_end: struct {
    ///         /// Is the document end indicator implicit?
    ///         implicit: i32,
    ///     },
    ///     /// The alias parameters (for YamlAliasEvent).
    ///     alias: struct {
    ///         /// The anchor.
    ///         anchor: *mut u8,
    ///     },
    ///     /// The scalar parameters (for YamlScalarEvent).
    ///     scalar: struct {
    ///         /// The anchor.
    ///         anchor: *mut u8,
    ///         /// The tag.
    ///         tag: *mut u8,
    ///         /// The scalar value.
    ///         value: *mut u8,
    ///         /// The length of the scalar value.
    ///         length: u64,
    ///         /// Is the tag optional for the plain style?
    ///         plain_implicit: i32,
    ///         /// Is the tag optional for any non-plain style?
    ///         quoted_implicit: i32,
    ///         /// The scalar style.
    ///         style: YamlScalarStyleT,
    ///     },
    ///     /// The sequence parameters (for YamlSequenceStartEvent).
    ///     sequence_start: struct {
    ///         /// The anchor.
    ///         anchor: *mut u8,
    ///         /// The tag.
    ///         tag: *mut u8,
    ///         /// Is the tag optional?
    ///         implicit: i32,
    ///         /// The sequence style.
    ///         style: YamlSequenceStyleT,
    ///     },
    ///     /// The mapping parameters (for YamlMappingStartEvent).
    ///     mapping_start: struct {
    ///         /// The anchor.
    ///         anchor: *mut u8,
    ///         /// The tag.
    ///         tag: *mut u8,
    ///         /// Is the tag optional?
    ///         implicit: i32,
    ///         /// The mapping style.
    ///         style: YamlMappingStyleT,
    ///     },
    /// }
    /// # };
    /// ```
    pub data: UnnamedYamlEventTData,
    /// The beginning of the event.
    pub start_mark: YamlMarkT,
    /// The end of the event.
    pub end_mark: YamlMarkT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct UnnamedYamlEventTData {
    /// The stream parameters (for YamlStreamStartEvent).
    pub stream_start: UnnamedYamlEventTdataStreamStart,
    /// The document parameters (for YamlDocumentStartEvent).
    pub document_start: UnnamedYamlEventTdataDocumentStart,
    /// The document end parameters (for YamlDocumentEndEvent).
    pub document_end: UnnamedYamlEventTdataDocumentEnd,
    /// The alias parameters (for YamlAliasEvent).
    pub alias: UnnamedYamlEventTdataAlias,
    /// The scalar parameters (for YamlScalarEvent).
    pub scalar: UnnamedYamlEventTdataScalar,
    /// The sequence parameters (for YamlSequenceStartEvent).
    pub sequence_start: UnnamedYamlEventTdataSequenceStart,
    /// The mapping parameters (for YamlMappingStartEvent).
    pub mapping_start: UnnamedYamlEventTdataMappingStart,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlEventTdataStreamStart {
    /// The document encoding.
    pub encoding: YamlEncodingT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlEventTdataDocumentStart {
    /// The version directive.
    pub version_directive: *mut YamlVersionDirectiveT,
    /// The list of tag directives.
    pub tag_directives: UnnamedYamlEventTdataDocumentStartTagDirectives,
    /// Is the document indicator implicit?
    pub implicit: bool,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlEventTdataDocumentStartTagDirectives {
    /// The beginning of the tag directives list.
    pub start: *mut YamlTagDirectiveT,
    /// The end of the tag directives list.
    pub end: *mut YamlTagDirectiveT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlEventTdataDocumentEnd {
    /// Is the document end indicator implicit?
    pub implicit: bool,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlEventTdataAlias {
    /// The anchor.
    pub anchor: *mut yaml_char_t,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlEventTdataScalar {
    /// The anchor.
    pub anchor: *mut yaml_char_t,
    /// The tag.
    pub tag: *mut yaml_char_t,
    /// The scalar value.
    pub value: *mut yaml_char_t,
    /// The length of the scalar value.
    pub length: size_t,
    /// Is the tag optional for the plain style?
    pub plain_implicit: bool,
    /// Is the tag optional for any non-plain style?
    pub quoted_implicit: bool,
    /// The scalar style.
    pub style: YamlScalarStyleT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlEventTdataSequenceStart {
    /// The anchor.
    pub anchor: *mut yaml_char_t,
    /// The tag.
    pub tag: *mut yaml_char_t,
    /// Is the tag optional?
    pub implicit: bool,
    /// The sequence style.
    pub style: YamlSequenceStyleT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlEventTdataMappingStart {
    /// The anchor.
    pub anchor: *mut yaml_char_t,
    /// The tag.
    pub tag: *mut yaml_char_t,
    /// Is the tag optional?
    pub implicit: bool,
    /// The mapping style.
    pub style: YamlMappingStyleT,
}

/// Node types.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
#[non_exhaustive]
pub enum YamlNodeTypeT {
    /// An empty node.
    YamlNoNode = 0,
    /// A scalar node.
    YamlScalarNode = 1,
    /// A sequence node.
    YamlSequenceNode = 2,
    /// A mapping node.
    YamlMappingNode = 3,
}

/// The node structure.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlNodeT {
    /// The node type.
    pub type_: YamlNodeTypeT,
    /// The node tag.
    pub tag: *mut yaml_char_t,
    /// The node data.
    ///
    /// ```
    /// # const _: &str = stringify! {
    /// union {
    ///     /// The scalar parameters (for YamlScalarNode).
    ///     scalar: struct {
    ///         /// The scalar value.
    ///         value: *mut u8,
    ///         /// The length of the scalar value.
    ///         length: u64,
    ///         /// The scalar style.
    ///         style: YamlScalarStyleT,
    ///     },
    ///     /// The sequence parameters (for YamlSequenceNode).
    ///     sequence: struct {
    ///         /// The stack of sequence items.
    ///         items: YamlStackT<YamlNodeItemT>,
    ///         /// The sequence style.
    ///         style: YamlSequenceStyleT,
    ///     },
    ///     /// The mapping parameters (for YamlMappingNode).
    ///     mapping: struct {
    ///         /// The stack of mapping pairs (key, value).
    ///         pairs: YamlStackT<YamlNodePairT>,
    ///         /// The mapping style.
    ///         style: YamlMappingStyleT,
    ///     },
    /// }
    /// # };
    /// ```
    pub data: UnnamedYamlNodeTData,
    /// The beginning of the node.
    pub start_mark: YamlMarkT,
    /// The end of the node.
    pub end_mark: YamlMarkT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct UnnamedYamlNodeTData {
    /// The scalar parameters (for YamlScalarNode).
    pub scalar: UnnamedYamlNodeTDataScalar,
    /// The sequence parameters (for YamlSequenceNode).
    pub sequence: UnnamedYamlNodeTDataSequence,
    /// The mapping parameters (for YamlMappingNode).
    pub mapping: UnnamedYamlNodeTDataMapping,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlNodeTDataScalar {
    /// The scalar value.
    pub value: *mut yaml_char_t,
    /// The length of the scalar value.
    pub length: size_t,
    /// The scalar style.
    pub style: YamlScalarStyleT,
}

/// An element of a sequence node.
pub type YamlNodeItemT = libc::c_int;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlNodeTDataSequence {
    /// The stack of sequence items.
    pub items: YamlStackT<YamlNodeItemT>,
    /// The sequence style.
    pub style: YamlSequenceStyleT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlNodeTDataMapping {
    /// The stack of mapping pairs (key, value).
    pub pairs: YamlStackT<YamlNodePairT>,
    /// The mapping style.
    pub style: YamlMappingStyleT,
}

/// An element of a mapping node.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlNodePairT {
    /// The key of the element.
    pub key: libc::c_int,
    /// The value of the element.
    pub value: libc::c_int,
}

/// The document structure.
#[derive(Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlDocumentT {
    /// The document nodes.
    pub nodes: YamlStackT<YamlNodeT>,
    /// The version directive.
    pub version_directive: *mut YamlVersionDirectiveT,
    /// The list of tag directives.
    ///
    /// ```
    /// # const _: &str = stringify! {
    /// struct {
    ///     /// The beginning of the tag directives list.
    ///     start: *mut YamlTagDirectiveT,
    ///     /// The end of the tag directives list.
    ///     end: *mut YamlTagDirectiveT,
    /// }
    /// # };
    /// ```
    pub tag_directives: UnnamedYamlDocumentTTagDirectives,
    /// Is the document start indicator implicit?
    pub start_implicit: bool,
    /// Is the document end indicator implicit?
    pub end_implicit: bool,
    /// The beginning of the document.
    pub start_mark: YamlMarkT,
    /// The end of the document.
    pub end_mark: YamlMarkT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct UnnamedYamlDocumentTTagDirectives {
    /// The beginning of the tag directives list.
    pub start: *mut YamlTagDirectiveT,
    /// The end of the tag directives list.
    pub end: *mut YamlTagDirectiveT,
}

/// The prototype of a read handler.
///
/// The read handler is called when the parser needs to read more bytes from the
/// source. The handler should write not more than `size` bytes to the `buffer`.
/// The number of written bytes should be set to the `length` variable.
///
/// On success, the handler should return 1. If the handler failed, the returned
/// value should be 0. On EOF, the handler should set the `size_read` to 0 and
/// return 1.
pub type YamlReadHandlerT = unsafe fn(
    data: *mut libc::c_void,
    buffer: *mut libc::c_uchar,
    size: size_t,
    size_read: *mut size_t,
) -> libc::c_int;

/// This structure holds information about a potential simple key.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlSimpleKeyT {
    /// Is a simple key possible?
    pub possible: bool,
    /// Is a simple key required?
    pub required: bool,
    /// The number of the token.
    pub token_number: size_t,
    /// The position mark.
    pub mark: YamlMarkT,
}

/// The states of the parser.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
#[non_exhaustive]
pub enum YamlParserStateT {
    /// Expect stream-start.
    YamlParseStreamStartState = 0,
    /// Expect the beginning of an implicit document.
    YamlParseImplicitDocumentStartState = 1,
    /// Expect document-start.
    YamlParseDocumentStartState = 2,
    /// Expect the content of a document.
    YamlParseDocumentContentState = 3,
    /// Expect document-end.
    YamlParseDocumentEndState = 4,
    /// Expect a block node.
    YamlParseBlockNodeState = 5,
    /// Expect a block node or indentless sequence.
    YamlParseBlockNodeOrIndentlessSequenceState = 6,
    /// Expect a flow node.
    YamlParseFlowNodeState = 7,
    /// Expect the first entry of a block sequence.
    YamlParseBlockSequenceFirstEntryState = 8,
    /// Expect an entry of a block sequence.
    YamlParseBlockSequenceEntryState = 9,
    /// Expect an entry of an indentless sequence.
    YamlParseIndentlessSequenceEntryState = 10,
    /// Expect the first key of a block mapping.
    YamlParseBlockMappingFirstKeyState = 11,
    /// Expect a block mapping key.
    YamlParseBlockMappingKeyState = 12,
    /// Expect a block mapping value.
    YamlParseBlockMappingValueState = 13,
    /// Expect the first entry of a flow sequence.
    YamlParseFlowSequenceFirstEntryState = 14,
    /// Expect an entry of a flow sequence.
    YamlParseFlowSequenceEntryState = 15,
    /// Expect a key of an ordered mapping.
    YamlParseFlowSequenceEntryMappingKeyState = 16,
    /// Expect a value of an ordered mapping.
    YamlParseFlowSequenceEntryMappingValueState = 17,
    /// Expect the and of an ordered mapping entry.
    YamlParseFlowSequenceEntryMappingEndState = 18,
    /// Expect the first key of a flow mapping.
    YamlParseFlowMappingFirstKeyState = 19,
    /// Expect a key of a flow mapping.
    YamlParseFlowMappingKeyState = 20,
    /// Expect a value of a flow mapping.
    YamlParseFlowMappingValueState = 21,
    /// Expect an empty value of a flow mapping.
    YamlParseFlowMappingEmptyValueState = 22,
    /// Expect nothing.
    YamlParseEndState = 23,
}

/// This structure holds aliases data.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlAliasDataT {
    /// The anchor.
    pub anchor: *mut yaml_char_t,
    /// The node id.
    pub index: libc::c_int,
    /// The anchor mark.
    pub mark: YamlMarkT,
}

/// The parser structure.
///
/// All members are internal. Manage the structure using the `yaml_parser_`
/// family of functions.
#[derive(Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlParserT {
    /// Error type.
    #[cfg(doc)]
    pub error: YamlErrorTypeT,
    #[cfg(not(doc))]
    pub(crate) error: YamlErrorTypeT,
    /// Error description.
    #[cfg(doc)]
    pub problem: *const libc::c_char,
    #[cfg(not(doc))]
    pub(crate) problem: *const libc::c_char,
    /// The byte about which the problem occurred.
    #[cfg(doc)]
    pub problem_offset: size_t,
    #[cfg(not(doc))]
    pub(crate) problem_offset: size_t,
    /// The problematic value (-1 is none).
    #[cfg(doc)]
    pub problem_value: libc::c_int,
    #[cfg(not(doc))]
    pub(crate) problem_value: libc::c_int,
    /// The problem position.
    #[cfg(doc)]
    pub problem_mark: YamlMarkT,
    #[cfg(not(doc))]
    pub(crate) problem_mark: YamlMarkT,
    /// The error context.
    #[cfg(doc)]
    pub context: *const libc::c_char,
    #[cfg(not(doc))]
    pub(crate) context: *const libc::c_char,
    /// The context position.
    #[cfg(doc)]
    pub context_mark: YamlMarkT,
    #[cfg(not(doc))]
    pub(crate) context_mark: YamlMarkT,
    /// Read handler.
    pub(crate) read_handler: Option<YamlReadHandlerT>,
    /// A pointer for passing to the read handler.
    pub(crate) read_handler_data: *mut libc::c_void,
    /// Standard (string or file) input data.
    pub(crate) input: UnnamedYamlParserTInput,
    /// EOF flag
    pub(crate) eof: bool,
    /// The working buffer.
    pub(crate) buffer: YamlBufferT<yaml_char_t>,
    /// The number of unread characters in the buffer.
    pub(crate) unread: size_t,
    /// The raw buffer.
    pub(crate) raw_buffer: YamlBufferT<libc::c_uchar>,
    /// The input encoding.
    pub(crate) encoding: YamlEncodingT,
    /// The offset of the current position (in bytes).
    pub(crate) offset: size_t,
    /// The mark of the current position.
    pub(crate) mark: YamlMarkT,
    /// Have we started to scan the input stream?
    pub(crate) stream_start_produced: bool,
    /// Have we reached the end of the input stream?
    pub(crate) stream_end_produced: bool,
    /// The number of unclosed '[' and '{' indicators.
    pub(crate) flow_level: libc::c_int,
    /// The tokens queue.
    pub(crate) tokens: YamlQueueT<YamlTokenT>,
    /// The number of tokens fetched from the queue.
    pub(crate) tokens_parsed: size_t,
    /// Does the tokens queue contain a token ready for dequeueing.
    pub(crate) token_available: bool,
    /// The indentation levels stack.
    pub(crate) indents: YamlStackT<libc::c_int>,
    /// The current indentation level.
    pub(crate) indent: libc::c_int,
    /// May a simple key occur at the current position?
    pub(crate) simple_key_allowed: bool,
    /// The stack of simple keys.
    pub(crate) simple_keys: YamlStackT<YamlSimpleKeyT>,
    /// At least this many leading elements of simple_keys have possible=0.
    pub(crate) not_simple_keys: libc::c_int,
    /// The parser states stack.
    pub(crate) states: YamlStackT<YamlParserStateT>,
    /// The current parser state.
    pub(crate) state: YamlParserStateT,
    /// The stack of marks.
    pub(crate) marks: YamlStackT<YamlMarkT>,
    /// The list of TAG directives.
    pub(crate) tag_directives: YamlStackT<YamlTagDirectiveT>,
    /// The alias data.
    pub(crate) aliases: YamlStackT<YamlAliasDataT>,
    /// The currently parsed document.
    pub(crate) document: *mut YamlDocumentT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlParserTPrefix {
    /// Error type.
    pub error: YamlErrorTypeT,
    /// Error description.
    pub problem: *const libc::c_char,
    /// The byte about which the problem occurred.
    pub problem_offset: size_t,
    /// The problematic value (-1 is none).
    pub problem_value: libc::c_int,
    /// The problem position.
    pub problem_mark: YamlMarkT,
    /// The error context.
    pub context: *const libc::c_char,
    /// The context position.
    pub context_mark: YamlMarkT,
}

#[doc(hidden)]
impl Deref for YamlParserT {
    type Target = YamlParserTPrefix;
    fn deref(&self) -> &Self::Target {
        unsafe { &*addr_of!(*self).cast() }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(crate) struct UnnamedYamlParserTInput {
    /// String input data.
    pub(crate) string: UnnamedYamlParserTInputString,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(crate) struct UnnamedYamlParserTInputString {
    /// The string start pointer.
    pub(crate) start: *const libc::c_uchar,
    /// The string end pointer.
    pub(crate) end: *const libc::c_uchar,
    /// The string current position.
    pub(crate) current: *const libc::c_uchar,
}

/// The prototype of a write handler.
///
/// The write handler is called when the emitter needs to flush the accumulated
/// characters to the output. The handler should write `size` bytes of the
/// `buffer` to the output.
///
/// On success, the handler should return 1. If the handler failed, the returned
/// value should be 0.
pub type YamlWriteHandlerT = unsafe fn(
    data: *mut libc::c_void,
    buffer: *mut libc::c_uchar,
    size: size_t,
) -> libc::c_int;

/// The emitter states.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u32)]
#[non_exhaustive]
pub enum YamlEmitterStateT {
    /// Expect stream-start.
    YamlEmitStreamStartState = 0,
    /// Expect the first document-start or stream-end.
    YamlEmitFirstDocumentStartState = 1,
    /// Expect document-start or stream-end.
    YamlEmitDocumentStartState = 2,
    /// Expect the content of a document.
    YamlEmitDocumentContentState = 3,
    /// Expect document-end.
    YamlEmitDocumentEndState = 4,
    /// Expect the first item of a flow sequence.
    YamlEmitFlowSequenceFirstItemState = 5,
    /// Expect an item of a flow sequence.
    YamlEmitFlowSequenceItemState = 6,
    /// Expect the first key of a flow mapping.
    YamlEmitFlowMappingFirstKeyState = 7,
    /// Expect a key of a flow mapping.
    YamlEmitFlowMappingKeyState = 8,
    /// Expect a value for a simple key of a flow mapping.
    YamlEmitFlowMappingSimpleValueState = 9,
    /// Expect a value of a flow mapping.
    YamlEmitFlowMappingValueState = 10,
    /// Expect the first item of a block sequence.
    YamlEmitBlockSequenceFirstItemState = 11,
    /// Expect an item of a block sequence.
    YamlEmitBlockSequenceItemState = 12,
    /// Expect the first key of a block mapping.
    YamlEmitBlockMappingFirstKeyState = 13,
    /// Expect the key of a block mapping.
    YamlEmitBlockMappingKeyState = 14,
    /// Expect a value for a simple key of a block mapping.
    YamlEmitBlockMappingSimpleValueState = 15,
    /// Expect a value of a block mapping.
    YamlEmitBlockMappingValueState = 16,
    /// Expect nothing.
    YamlEmitEndState = 17,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(crate) struct YamlAnchorsT {
    /// The number of references.
    pub(crate) references: libc::c_int,
    /// The anchor id.
    pub(crate) anchor: libc::c_int,
    /// If the node has been emitted?
    pub(crate) serialized: bool,
}

/// The emitter structure.
///
/// All members are internal. Manage the structure using the `yaml_emitter_`
/// family of functions.
#[derive(Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlEmitterT {
    /// Error type.
    #[cfg(doc)]
    pub error: YamlErrorTypeT,
    #[cfg(not(doc))]
    pub(crate) error: YamlErrorTypeT,
    /// Error description.
    #[cfg(doc)]
    pub problem: *const libc::c_char,
    #[cfg(not(doc))]
    pub(crate) problem: *const libc::c_char,
    /// Write handler.
    pub(crate) write_handler: Option<YamlWriteHandlerT>,
    /// A pointer for passing to the write handler.
    pub(crate) write_handler_data: *mut libc::c_void,
    /// Standard (string or file) output data.
    pub(crate) output: UnnamedYamlEmitterTOutput,
    /// The working buffer.
    pub(crate) buffer: YamlBufferT<yaml_char_t>,
    /// The raw buffer.
    pub(crate) raw_buffer: YamlBufferT<libc::c_uchar>,
    /// The stream encoding.
    pub(crate) encoding: YamlEncodingT,
    /// If the output is in the canonical style?
    pub(crate) canonical: bool,
    /// The number of indentation spaces.
    pub(crate) best_indent: libc::c_int,
    /// The preferred width of the output lines.
    pub(crate) best_width: libc::c_int,
    /// Allow unescaped non-ASCII characters?
    pub(crate) unicode: bool,
    /// The preferred line break.
    pub(crate) line_break: YamlBreakT,
    /// The stack of states.
    pub(crate) states: YamlStackT<YamlEmitterStateT>,
    /// The current emitter state.
    pub(crate) state: YamlEmitterStateT,
    /// The event queue.
    pub(crate) events: YamlQueueT<YamlEventT>,
    /// The stack of indentation levels.
    pub(crate) indents: YamlStackT<libc::c_int>,
    /// The list of tag directives.
    pub(crate) tag_directives: YamlStackT<YamlTagDirectiveT>,
    /// The current indentation level.
    pub(crate) indent: libc::c_int,
    /// The current flow level.
    pub(crate) flow_level: libc::c_int,
    /// Is it the document root context?
    pub(crate) root_context: bool,
    /// Is it a sequence context?
    pub(crate) sequence_context: bool,
    /// Is it a mapping context?
    pub(crate) mapping_context: bool,
    /// Is it a simple mapping key context?
    pub(crate) simple_key_context: bool,
    /// The current line.
    pub(crate) line: libc::c_int,
    /// The current column.
    pub(crate) column: libc::c_int,
    /// If the last character was a whitespace?
    pub(crate) whitespace: bool,
    /// If the last character was an indentation character (' ', '-', '?', ':')?
    pub(crate) indention: bool,
    /// If an explicit document end is required?
    pub(crate) open_ended: libc::c_int,
    /// Anchor analysis.
    pub(crate) anchor_data: UnnamedYamlEmitterTAnchorData,
    /// Tag analysis.
    pub(crate) tag_data: UnnamedYamlEmitterTTagData,
    /// Scalar analysis.
    pub(crate) scalar_data: UnnamedYamlEmitterTScalarData,
    /// If the stream was already opened?
    pub(crate) opened: bool,
    /// If the stream was already closed?
    pub(crate) closed: bool,
    /// The information associated with the document nodes.
    pub(crate) anchors: *mut YamlAnchorsT,
    /// The last assigned anchor id.
    pub(crate) last_anchor_id: libc::c_int,
    /// The currently emitted document.
    pub(crate) document: *mut YamlDocumentT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
#[non_exhaustive]
pub struct YamlEmitterTPrefix {
    /// Error type.
    pub error: YamlErrorTypeT,
    /// Error description.
    pub problem: *const libc::c_char,
}

#[doc(hidden)]
impl Deref for YamlEmitterT {
    type Target = YamlEmitterTPrefix;
    fn deref(&self) -> &Self::Target {
        unsafe { &*addr_of!(*self).cast() }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(crate) struct UnnamedYamlEmitterTOutput {
    /// String output data.
    pub(crate) string: UnnamedYamlEmitterTOutputString,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(crate) struct UnnamedYamlEmitterTOutputString {
    /// The buffer pointer.
    pub(crate) buffer: *mut libc::c_uchar,
    /// The buffer size.
    pub(crate) size: size_t,
    /// The number of written bytes.
    pub(crate) size_written: *mut size_t,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(crate) struct UnnamedYamlEmitterTAnchorData {
    /// The anchor value.
    pub(crate) anchor: *mut yaml_char_t,
    /// The anchor length.
    pub(crate) anchor_length: size_t,
    /// Is it an alias?
    pub(crate) alias: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub(crate) struct UnnamedYamlEmitterTTagData {
    /// The tag handle.
    pub(crate) handle: *mut yaml_char_t,
    /// The tag handle length.
    pub(crate) handle_length: size_t,
    /// The tag suffix.
    pub(crate) suffix: *mut yaml_char_t,
    /// The tag suffix length.
    pub(crate) suffix_length: size_t,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(crate) struct UnnamedYamlEmitterTScalarData {
    /// The scalar value.
    pub(crate) value: *mut yaml_char_t,
    /// The scalar length.
    pub(crate) length: size_t,
    /// Does the scalar contain line breaks?
    pub(crate) multiline: bool,
    /// Can the scalar be expressed in the flow plain style?
    pub(crate) flow_plain_allowed: bool,
    /// Can the scalar be expressed in the block plain style?
    pub(crate) block_plain_allowed: bool,
    /// Can the scalar be expressed in the single quoted style?
    pub(crate) single_quoted_allowed: bool,
    /// Can the scalar be expressed in the literal or folded styles?
    pub(crate) block_allowed: bool,
    /// The output style.
    pub(crate) style: YamlScalarStyleT,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(crate) struct YamlStringT {
    pub(crate) start: *mut yaml_char_t,
    pub(crate) end: *mut yaml_char_t,
    pub(crate) pointer: *mut yaml_char_t,
}

pub(crate) const NULL_STRING: YamlStringT = YamlStringT {
    start: ptr::null_mut::<yaml_char_t>(),
    end: ptr::null_mut::<yaml_char_t>(),
    pointer: ptr::null_mut::<yaml_char_t>(),
};

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub(crate) struct YamlBufferT<T> {
    /// The beginning of the buffer.
    pub(crate) start: *mut T,
    /// The end of the buffer.
    pub(crate) end: *mut T,
    /// The current position of the buffer.
    pub(crate) pointer: *mut T,
    /// The last filled position of the buffer.
    pub(crate) last: *mut T,
}

impl<T> YamlBufferT<T> {
    /// Is the buffer empty?
    pub(crate) fn is_empty(&self) -> bool {
        self.pointer == self.last
    }

    /// Advances the buffer by one character.
    pub(crate) fn next(&mut self) {
        if !self.is_empty() {
            unsafe {
                self.pointer = self.pointer.add(1);
            }
        }
    }
}

// impl<T> Copy for YamlBufferT<T> {}
// impl<T> Clone for YamlBufferT<T> {
//     fn clone(&self) -> Self {
//         *self
//     }
// }

#[derive(Debug)]
#[repr(C)]
/// The beginning of the stack.
pub struct YamlStackT<T> {
    /// The beginning of the stack.
    pub start: *mut T,
    /// The end of the stack.
    pub end: *mut T,
    /// The top of the stack.
    pub top: *mut T,
}

impl<T> Copy for YamlStackT<T> {}
impl<T> Clone for YamlStackT<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[derive(Debug)]
#[repr(C)]
pub(crate) struct YamlQueueT<T> {
    /// The beginning of the queue.
    pub(crate) start: *mut T,
    /// The end of the queue.
    pub(crate) end: *mut T,
    /// The head of the queue.
    pub(crate) head: *mut T,
    /// The tail of the queue.
    pub(crate) tail: *mut T,
}

impl<T> Copy for YamlQueueT<T> {}
impl<T> Clone for YamlQueueT<T> {
    fn clone(&self) -> Self {
        *self
    }
}
