//! Repairs text that was decoded with the wrong character set on the way into
//! the clipboard.
//!
//! Some remote-desktop / HTML-clipboard producers write raw UTF-8 bytes into a
//! `CF_UNICODETEXT` slot. `clipboard-rs` reads that format as UTF-16LE, so every
//! pair of UTF-8 bytes turns into one CJK-looking char (`Ve` -> U+6556, `rs` ->
//! U+7372, ...). The result is a valid Rust string full of rare ideographs, so
//! no amount of UTF-8 validation at the call site can catch it.
//!
//! The reverse transform is unambiguous: encode back to UTF-16LE bytes and try
//! a *strict* UTF-8 decode. The guard rails below only accept the repair when
//! the recovered text is dramatically more "normal" than what we received,
//! which keeps genuine CJK text (whose UTF-16LE bytes are not valid UTF-8) and
//! plain ASCII (which would produce embedded NULs) from being rewritten.

/// Characters expected in ordinary clipboard text: ASCII, common whitespace,
/// CJK Unified Ideographs, CJK punctuation/symbols and full-width forms.
fn is_common(ch: char) -> bool {
    let code = ch as u32;
    ch.is_ascii_graphic()
        || matches!(ch, ' ' | '\r' | '\n' | '\t')
        || (0x4E00..=0x9FFF).contains(&code)
        || (0x3000..=0x303F).contains(&code)
        || (0xFF00..=0xFFEF).contains(&code)
}

fn common_ratio(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }
    let common = text.chars().filter(|ch| is_common(*ch)).count();
    common as f64 / text.chars().count() as f64
}

/// Returns the repaired text when `text` is almost certainly a UTF-8 payload
/// that was read as UTF-16LE. `None` means "leave the original untouched".
///
/// Conditions, all of which must hold:
/// 1. re-encoding to UTF-16LE yields a byte string that is strictly valid UTF-8;
/// 2. the decoded text contains no NUL / control characters beyond whitespace;
/// 3. the decoded text is made up almost entirely of common characters;
/// 4. the original contained a significant share of non-common characters —
///    genuine text (ASCII or Chinese) already scores ~1.0 and is never touched.
pub fn repair_mojibake(text: &str) -> Option<String> {
    if text.chars().count() < 8 || common_ratio(text) >= 0.95 {
        return None;
    }

    let mut bytes = Vec::with_capacity(text.len() * 2);
    for unit in text.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    let decoded = String::from_utf8(bytes).ok()?;
    if decoded.chars().count() < 8 {
        return None;
    }
    if decoded
        .chars()
        .any(|ch| ch.is_control() && !matches!(ch, '\r' | '\n' | '\t'))
    {
        return None;
    }
    if common_ratio(&decoded) < 0.99 {
        return None;
    }
    Some(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repairs_utf8_read_as_utf16le() {
        // The actual prefix of the weibo CF_HTML fragment saved in history.
        let garbled = "敖獲潩㩮⸰ഹ匊慴瑲呈䱍〺〰〰〰㔱";
        let repaired = repair_mojibake(garbled).expect("garbled text should be repaired");
        assert!(repaired.starts_with("Version:0.9\r\nStartHTML:"));
    }

    #[test]
    fn leaves_ascii_alone() {
        // ASCII re-encoded as UTF-16LE decodes cleanly but is full of NULs.
        assert!(repair_mojibake("SELECT * FROM clipboard_history WHERE id = 42").is_none());
    }

    #[test]
    fn leaves_chinese_alone() {
        assert!(repair_mojibake("标准计划保险方案介绍：责任项目名称、保险责任简要描述").is_none());
    }

    #[test]
    fn leaves_short_text_alone() {
        assert!(repair_mojibake("敖獲潩㩮").is_none());
    }

    #[test]
    fn repaired_text_is_mostly_common() {
        let garbled = "敖獲潩㩮⸰ഹ匊慴瑲呈䱍〺〰〰〰㔱";
        let repaired = repair_mojibake(garbled).unwrap();
        assert!(common_ratio(&repaired) >= 0.99);
    }
}
