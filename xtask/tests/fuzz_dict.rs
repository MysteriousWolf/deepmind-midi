//! The fuzz dictionary has to follow the protocol.
//!
//! `fuzz/deepmind.dict` hands the fuzzer the tokens it cannot guess: the frame
//! header and the command bytes. A command the library learns without a token
//! here is one the fuzzer reaches only by luck, so this reads the dictionary
//! back and checks it against the library's own tables.

use std::path::{Path, PathBuf};

use deepmind_midi::ids::{MANUFACTURER_ID, Model};
use deepmind_midi::sysex::Command;

fn dictionary_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fuzz/deepmind.dict")
}

/// Reads the dictionary's tokens as byte strings, in file order.
fn tokens() -> Vec<Vec<u8>> {
    let path = dictionary_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()));
    let tokens: Vec<Vec<u8>> = text.lines().filter_map(parse_line).collect();
    assert!(!tokens.is_empty(), "{} holds no tokens", path.display());
    tokens
}

/// Parses one `name="value"` line. Comments and blank lines are `None`.
fn parse_line(line: &str) -> Option<Vec<u8>> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let start = line.find('"')?;
    let end = line.rfind('"')?;
    assert!(end > start, "malformed dictionary line: {line}");
    Some(unescape(&line[start + 1..end]))
}

/// Decodes libFuzzer's escapes: `\xHH`, `\\` and `\"`.
fn unescape(value: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let mut bytes = value.bytes();
    while let Some(byte) = bytes.next() {
        if byte != b'\\' {
            out.push(byte);
            continue;
        }
        match bytes.next() {
            Some(b'x') => {
                let hex: String = (0..2)
                    .filter_map(|_| bytes.next())
                    .map(char::from)
                    .collect();
                let decoded = u8::from_str_radix(&hex, 16)
                    .unwrap_or_else(|error| panic!("bad escape \\x{hex} in {value:?}: {error}"));
                out.push(decoded);
            }
            Some(other) => out.push(other),
            None => panic!("dangling backslash in {value:?}"),
        }
    }
    out
}

#[test]
fn every_command_has_a_token() {
    let tokens = tokens();
    for command in Command::ALL.iter().copied() {
        let byte = command.to_byte();
        assert!(
            tokens.iter().any(|token| token.as_slice() == [byte]),
            "fuzz/deepmind.dict has no token for {command} ({byte:#04X}); \
             add one so the fuzzer can reach it"
        );
    }
}

#[test]
fn the_header_token_is_the_protocol_header() {
    let tokens = tokens();
    let mut header = vec![0xF0];
    header.extend(MANUFACTURER_ID);
    header.push(Model::MODEL_ID);
    assert!(
        tokens.contains(&header),
        "fuzz/deepmind.dict has no token for the frame header {header:02X?}"
    );
}
