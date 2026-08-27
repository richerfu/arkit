//! VT samples mirrored from `@ohos-rs/terminal` `example/src/main/ets/pages/Index.ets`.

/// 8 ANSI colors, SGR attributes, underline styles, and emoji.
pub const COLORS: &str = "\
\x1b[0m\
\x1b[30mblk\x1b[0m \x1b[31mred\x1b[0m \x1b[32mgrn\x1b[0m \x1b[33mylw\x1b[0m \
\x1b[34mblu\x1b[0m \x1b[35mmag\x1b[0m \x1b[36mcyn\x1b[0m \x1b[37mwht\x1b[0m\r\n\
\x1b[1mbold\x1b[0m \x1b[3mitalic\x1b[0m \x1b[2mfaint\x1b[0m \
\x1b[7minverse\x1b[0m \x1b[9mstrike\x1b[0m \x1b[53moverline\x1b[0m\r\n\
\x1b[4msingle\x1b[0m \x1b[4:2mdouble\x1b[0m \x1b[4:3;58;2;255;100;80mcurly\x1b[0m \
\x1b[4:4mdotted\x1b[0m \x1b[4:5mdashed\x1b[0m  color emoji: 🙂 🚀\r\n";

pub const HELLO_COMMAND: &[u8] = b"echo hello from HarmonyOS\r";

/// Soft-wrap + Unicode + scrollback. 1,200 lines, same as the ETS demo.
pub fn stress_transcript(line_count: usize) -> String {
    let mut out = String::with_capacity(line_count * 140);
    out.push_str("\r\n\x1b[1;36m— long output / wrap / Unicode stress —\x1b[0m\r\n");
    for index in 0..line_count {
        let color = 31 + index % 6;
        let _ = std::fmt::Write::write_fmt(
            &mut out,
            format_args!(
                "\x1b[{color}m{index:04}\x1b[0m 0123456789 abcdefghijklmnopqrstuvwxyz \
                 soft-wrap-without-manual-column-break 中文 é 🙂 terminal-scrollback-validation\r\n"
            ),
        );
    }
    out.push_str("\x1b[1;32m— stress output complete; swipe to inspect history —\x1b[0m\r\n");
    out
}

#[cfg(test)]
mod tests {
    use super::stress_transcript;

    #[test]
    fn stress_emits_requested_line_count() {
        let text = stress_transcript(12);
        assert!(text.contains("0011"));
        assert!(text.contains("中文"));
        assert!(text.contains("🙂"));
    }
}
