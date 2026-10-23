use std::cmp::min;

pub struct Utf8Error;

pub fn trim_whitespace_start(mut s: &[u8]) -> Result<&[u8], Utf8Error> {
    const BLOCK_SIZE: usize = 16;
    while !s.is_empty() {
        let prefix = &s[..min(s.len(), BLOCK_SIZE)];
        let chunk = prefix.utf8_chunks().next().unwrap();
        let valid = chunk.valid();
        if valid.is_empty() {
            return Err(Utf8Error);
        }
        let trim_len = valid.len() - valid.trim_start().len();
        s = &s[trim_len..];
        if trim_len < valid.len() {
            break;
        }
    }
    Ok(s)
}

// A UTF-8 continuation byte matches the pattern 10xxxxxx.
fn is_continuation_byte(c: u8) -> bool {
    c & 0xc0 == 0x80
}

fn skip_continuation_bytes(mut s: &[u8]) -> &[u8] {
    while let Some((&c, rest)) = s.split_first()
        && is_continuation_byte(c)
    {
        s = rest;
    }
    s
}

pub fn trim_whitespace_end(mut s: &[u8]) -> Result<&[u8], Utf8Error> {
    const BLOCK_SIZE: usize = 16;
    while !s.is_empty() {
        let suffix = &s[s.len().saturating_sub(BLOCK_SIZE)..];
        let suffix = skip_continuation_bytes(suffix);
        if suffix.is_empty() {
            return Err(Utf8Error);
        }
        let Ok(suffix) = str::from_utf8(suffix) else {
            return Err(Utf8Error);
        };
        let trim_len = suffix.len() - suffix.trim_end().len();
        s = &s[..s.len() - trim_len];
        if trim_len < suffix.len() {
            break;
        }
    }
    Ok(s)
}

pub fn trim_whitespace(s: &[u8]) -> Result<&[u8], Utf8Error> {
    trim_whitespace_start(s).and_then(trim_whitespace_end)
}
