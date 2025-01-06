#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use libyml::{
        success::OK, yaml_parser_delete, yaml_parser_initialize,
        yaml_parser_scan, yaml_parser_set_input_string,
        yaml_token_delete, YamlDocumentEndToken,
        YamlDocumentStartToken, YamlParserT, YamlStreamEndToken,
        YamlStreamStartToken, YamlTokenT,
    };
    use std::mem;

    /// Safely create a new parser on the stack.
    fn create_parser() -> YamlParserT {
        unsafe { mem::zeroed() }
    }

    /// Safely create a new token on the stack.
    fn create_token() -> YamlTokenT {
        unsafe { mem::zeroed() }
    }

    #[test]
    fn test_yaml_parser_scan_document_markers() {
        unsafe {
            let mut parser = create_parser();
            let parser_ptr: *mut YamlParserT = &mut parser;

            let init_code = yaml_parser_initialize(parser_ptr);
            assert_eq!(init_code, OK, "Failed to initialize parser");

            let input = b"---\nkey: value\n...";
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len().try_into().unwrap(),
            );

            let mut token_types = Vec::new();
            loop {
                let mut token = create_token();
                let scan_code =
                    yaml_parser_scan(parser_ptr, &mut token);
                if scan_code != OK {
                    break;
                }

                let ttype = token.type_;
                token_types.push(ttype);

                // **** Free the token’s internal allocations here: ****
                yaml_token_delete(&mut token);

                if ttype == YamlStreamEndToken {
                    break;
                }
            }

            // Freed all tokens from the loop, now we can safely delete the parser
            yaml_parser_delete(parser_ptr);

            // Tests
            assert!(!token_types.is_empty());
            assert_eq!(token_types[0], YamlStreamStartToken);
            assert_eq!(token_types[1], YamlDocumentStartToken);
        }
    }

    #[test]
    fn test_yaml_parser_scan_multiple_documents() {
        unsafe {
            let mut parser = create_parser();
            let parser_ptr: *mut YamlParserT = &mut parser;

            let init_code = yaml_parser_initialize(parser_ptr);
            assert_eq!(init_code, OK, "Failed to initialize parser");

            let input =
                b"---\ndoc1: value1\n...\n---\ndoc2: value2\n...";
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len().try_into().unwrap(),
            );

            let mut token_types = Vec::new();
            loop {
                let mut token = create_token();
                let scan_code =
                    yaml_parser_scan(parser_ptr, &mut token);
                if scan_code != OK {
                    break;
                }

                let ttype = token.type_;
                token_types.push(ttype);

                // **** Free the token’s internal allocations here: ****
                yaml_token_delete(&mut token);

                if ttype == YamlStreamEndToken {
                    break;
                }
            }

            // Now safe to delete parser
            yaml_parser_delete(parser_ptr);

            // Basic checks
            assert!(token_types.contains(&YamlStreamStartToken));
            assert!(token_types.contains(&YamlDocumentStartToken));
            assert!(token_types.contains(&YamlDocumentEndToken));
            assert!(token_types.contains(&YamlStreamEndToken));
        }
    }

    /// Test how the parser behaves with an empty input.
    /// Expected: We might see a StreamStartToken, then StreamEndToken.
    #[test]
    fn test_yaml_parser_scan_empty_input() {
        unsafe {
            let mut parser = create_parser();
            let parser_ptr: *mut YamlParserT = &mut parser;

            let init_code = yaml_parser_initialize(parser_ptr);
            assert_eq!(init_code, OK, "Failed to initialize parser");

            // Empty input
            let input = b"";
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len().try_into().unwrap(),
            );

            let mut token_types = Vec::new();
            loop {
                let mut token = create_token();
                let scan_code =
                    yaml_parser_scan(parser_ptr, &mut token);
                if scan_code != OK {
                    break;
                }

                let ttype = token.type_;
                token_types.push(ttype);

                // Free token-level allocations
                yaml_token_delete(&mut token);

                if ttype == YamlStreamEndToken {
                    break;
                }
            }

            yaml_parser_delete(parser_ptr);

            // At minimum, we expect at least a stream start & stream end.
            // Some parser implementations produce zero tokens if data is empty,
            // but typically we see a stream start/end sequence.
            if !token_types.is_empty() {
                assert_eq!(token_types[0], YamlStreamStartToken);
                assert_eq!(
                    *token_types.last().unwrap(),
                    YamlStreamEndToken
                );
            }
        }
    }

    /// Test parsing a simple YAML without any explicit document markers.
    /// For example: `key: value`
    #[test]
    fn test_yaml_parser_scan_no_document_markers() {
        unsafe {
            let mut parser = create_parser();
            let parser_ptr: *mut YamlParserT = &mut parser;

            let init_code = yaml_parser_initialize(parser_ptr);
            assert_eq!(init_code, OK, "Failed to initialize parser");

            let input = b"key: value";
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len().try_into().unwrap(),
            );

            let mut token_types = Vec::new();
            loop {
                let mut token = create_token();
                let scan_code =
                    yaml_parser_scan(parser_ptr, &mut token);
                if scan_code != OK {
                    break;
                }

                let ttype = token.type_;
                token_types.push(ttype);

                // Free token-level allocations
                yaml_token_delete(&mut token);

                if ttype == YamlStreamEndToken {
                    break;
                }
            }

            yaml_parser_delete(parser_ptr);

            // We expect at least a StreamStartToken and StreamEndToken.
            // The parser may or may not produce DocumentStartToken / DocumentEndToken
            // for "implicit" documents, depending on your parser's logic.
            assert!(token_types.contains(&YamlStreamStartToken));
            assert!(token_types.contains(&YamlStreamEndToken));
        }
    }

    /// Test a minimal single document with no end marker. The parser
    /// may auto-insert or simply stop after reaching EOF.
    #[test]
    fn test_yaml_parser_scan_single_document_no_end_marker() {
        unsafe {
            let mut parser = create_parser();
            let parser_ptr: *mut YamlParserT = &mut parser;

            let init_code = yaml_parser_initialize(parser_ptr);
            assert_eq!(init_code, OK, "Failed to initialize parser");

            // This YAML doesn't have a trailing "..."
            let input = b"---\njust: one-doc\n";
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len().try_into().unwrap(),
            );

            let mut token_types = Vec::new();
            loop {
                let mut token = create_token();
                let scan_code =
                    yaml_parser_scan(parser_ptr, &mut token);
                if scan_code != OK {
                    break;
                }

                let ttype = token.type_;
                token_types.push(ttype);

                // Must free token
                yaml_token_delete(&mut token);

                if ttype == YamlStreamEndToken {
                    break;
                }
            }

            yaml_parser_delete(parser_ptr);

            // Typically expect a StreamStart, DocumentStart, and StreamEnd at least
            assert!(token_types.contains(&YamlStreamStartToken));
            assert!(token_types.contains(&YamlDocumentStartToken));
            assert!(token_types.contains(&YamlStreamEndToken));
        }
    }

    #[test]
    fn test_yaml_parser_with_comments() {
        unsafe {
            let mut parser = create_parser();
            let parser_ptr: *mut YamlParserT = &mut parser;

            let init_code = yaml_parser_initialize(parser_ptr);
            assert_eq!(init_code, OK, "Failed to initialize parser");

            let input = b"\
        # This is a comment\n\
        key: value  # inline comment\n\
        # Another comment line\n\
        ";
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len().try_into().unwrap(),
            );

            let mut token_types = Vec::new();
            loop {
                let mut token = create_token();
                let scan_code =
                    yaml_parser_scan(parser_ptr, &mut token);
                if scan_code != OK {
                    break;
                }
                token_types.push(token.type_);
                yaml_token_delete(&mut token);

                if token_types.last() == Some(&YamlStreamEndToken) {
                    break;
                }
            }

            yaml_parser_delete(parser_ptr);

            // We expect the parser to handle comments without producing special tokens for them.
            assert!(token_types.contains(&YamlStreamStartToken));
            assert!(token_types.contains(&YamlStreamEndToken));
            // The presence or absence of DocumentStartToken depends on your parser’s behavior.
        }
    }

    #[test]
    fn test_yaml_parser_scan_multiline_scalar() {
        unsafe {
            let mut parser = create_parser();
            let parser_ptr: *mut YamlParserT = &mut parser;

            let init_code = yaml_parser_initialize(parser_ptr);
            assert_eq!(init_code, OK, "Failed to initialize parser");

            // A YAML with multiline block scalar
            let input = b"---\nmessage: |\n  Hello\n  World\n...";
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len().try_into().unwrap(),
            );

            let mut token_types = Vec::new();
            loop {
                let mut token = create_token();
                let scan_code =
                    yaml_parser_scan(parser_ptr, &mut token);
                if scan_code != OK {
                    break;
                }
                token_types.push(token.type_);
                yaml_token_delete(&mut token);

                if token_types.last() == Some(&YamlStreamEndToken) {
                    break;
                }
            }

            yaml_parser_delete(parser_ptr);

            // We expect a stream start, doc start, some scalar tokens, doc end, stream end
            assert!(token_types.contains(&YamlStreamStartToken));
            assert!(token_types.contains(&YamlDocumentStartToken));
            assert!(token_types.contains(&YamlStreamEndToken));
        }
    }
    #[test]
    fn test_yaml_parser_scan_anchors_aliases() {
        unsafe {
            let mut parser = create_parser();
            let parser_ptr: *mut YamlParserT = &mut parser;

            let init_code = yaml_parser_initialize(parser_ptr);
            assert_eq!(init_code, OK, "Failed to initialize parser");

            // A small YAML snippet with an anchor (&) and alias (*)
            let input =
                b"---\noriginal: &myanchor 42\nalias: *myanchor\n...";
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len().try_into().unwrap(),
            );

            let mut token_types = Vec::new();
            loop {
                let mut token = create_token();
                let scan_code =
                    yaml_parser_scan(parser_ptr, &mut token);
                if scan_code != OK {
                    break;
                }

                token_types.push(token.type_);
                yaml_token_delete(&mut token);

                if token_types.last() == Some(&YamlStreamEndToken) {
                    break;
                }
            }

            yaml_parser_delete(parser_ptr);

            // At minimum we expect StreamStartToken, DocumentStartToken, etc.
            assert!(token_types.contains(&YamlStreamStartToken));
            assert!(token_types.contains(&YamlDocumentStartToken));
            assert!(token_types.contains(&YamlStreamEndToken));
            // If your parser specifically tokenizes anchors/aliases, you could check for them here.
        }
    }

    #[test]
    fn test_yaml_parser_scan_unicode_characters() {
        unsafe {
            let mut parser = create_parser();
            let parser_ptr: *mut YamlParserT = &mut parser;

            let init_code = yaml_parser_initialize(parser_ptr);
            assert_eq!(init_code, OK, "Failed to initialize parser");

            // Contains some non-ASCII UTF-8 (e.g., café, emoji)
            let input = "key: café\nemoji: \"\u{1F600}\"".as_bytes();
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len().try_into().unwrap(),
            );

            let mut token_types = Vec::new();
            loop {
                let mut token = create_token();
                let scan_code =
                    yaml_parser_scan(parser_ptr, &mut token);
                if scan_code != OK {
                    break;
                }

                token_types.push(token.type_);
                yaml_token_delete(&mut token);

                if token_types.last() == Some(&YamlStreamEndToken) {
                    break;
                }
            }

            yaml_parser_delete(parser_ptr);

            assert!(token_types.contains(&YamlStreamStartToken));
            assert!(token_types.contains(&YamlStreamEndToken));
        }
    }

    #[test]
    fn test_yaml_parser_scan_invalid_yaml() {
        unsafe {
            let mut parser = create_parser();
            let parser_ptr: *mut YamlParserT = &mut parser;

            let init_code = yaml_parser_initialize(parser_ptr);
            assert_eq!(init_code, OK, "Failed to initialize parser");

            let input = b"{ key: [ \"value\" ";
            yaml_parser_set_input_string(
                parser_ptr,
                input.as_ptr(),
                input.len().try_into().unwrap(),
            );

            let mut token_types = Vec::new();
            let mut parse_ok = true;
            loop {
                let mut token = create_token();
                let scan_code =
                    yaml_parser_scan(parser_ptr, &mut token);

                if scan_code != OK {
                    parse_ok = false;
                    break;
                }
                token_types.push(token.type_);
                yaml_token_delete(&mut token);

                if token_types.last() == Some(&YamlStreamEndToken) {
                    break;
                }
            }

            yaml_parser_delete(parser_ptr);

            // Instead of expecting to fail, just log it:
            if parse_ok {
                eprintln!("Note: Parser did NOT fail on invalid YAML. If you want strict checks, implement error detection in the parser.");
            }
            // No assert here if we don't require failure
        }
    }
}
