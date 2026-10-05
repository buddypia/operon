use crate::prelude::*;

pub(crate) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
/// SHA-256 as FIPS 180-4 describes it. Written out rather than taken from a
/// crate because the one caller — the trust hash Codex expects beside a hook
/// entry — is not worth a dependency, and a dependency is a paused surface.
pub(crate) fn sha256(bytes: &[u8]) -> [u8; 32] {
    const ROUND_CONSTANTS: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut message = bytes.to_vec();
    let bit_length = (bytes.len() as u64).wrapping_mul(8);
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_length.to_be_bytes());
    for block in message.chunks_exact(64) {
        let mut schedule = [0u32; 64];
        for (index, word) in block.chunks_exact(4).enumerate() {
            schedule[index] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for index in 16..64 {
            let near = schedule[index - 15];
            let far = schedule[index - 2];
            let sigma0 = near.rotate_right(7) ^ near.rotate_right(18) ^ (near >> 3);
            let sigma1 = far.rotate_right(17) ^ far.rotate_right(19) ^ (far >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(sigma0)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(sigma1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for (round, constant) in ROUND_CONSTANTS.iter().enumerate() {
            let big_sigma1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ (!e & g);
            let temp1 = h
                .wrapping_add(big_sigma1)
                .wrapping_add(choice)
                .wrapping_add(*constant)
                .wrapping_add(schedule[round]);
            let big_sigma0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = big_sigma0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut digest = [0u8; 32];
    for (index, word) in state.iter().enumerate() {
        digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    sha256(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The text a terminal would show, with the control sequences taken out.
///
/// `tmux capture-pane -e` keeps the escapes, which is what the terminal pane in
/// `src/ui/terminal.rs` wants and what any *reader* of a capture has to undo
/// first. Claude Code paints a selected menu line one word at a time, so
/// `No, exit` reaches a caller as `ESC[38;5;153mNo,ESC[39m ESC[38;5;153mexit`
/// and matching it as a substring finds nothing.
///
/// Two families, because a real capture carries both:
///
/// - **CSI** — `ESC [`, parameters, then one byte in `0x40..=0x7e`. Colour is
///   the common case, but cursor moves and erases come through the same door.
/// - **OSC** — `ESC ]`, a string, then `BEL` or `ESC \`. Claude Code wraps its
///   "Security guide" link in one, and a stripper written only for colour
///   leaves `]8;id=…;https://…` sitting in the middle of the text.
///
/// Anything else after `ESC` drops the `ESC` alone and keeps the byte after it.
/// That is deliberately the timid choice: an unrecognised sequence leaks one or
/// two characters into the text, where guessing at its length would eat words
/// that were on screen. Leaking is visible in a failing match; eating is not.
///
/// A sequence cut off by the end of the input is dropped rather than emitted —
/// a capture taken while a CLI is mid-write can halve one, and half an escape
/// is not text either.
pub(crate) fn strip_terminal_escapes(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut plain = String::with_capacity(text.len());
    let mut copied = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != 0x1b {
            index += 1;
            continue;
        }
        let end = match bytes.get(index + 1) {
            Some(b'[') => bytes[index + 2..]
                .iter()
                .position(|byte| (0x40..=0x7e).contains(byte))
                .map(|offset| index + 2 + offset + 1),
            Some(b']') => operating_system_command_end(bytes, index + 2),
            Some(b'\\') => Some(index + 2),
            // The three-byte escapes: `ESC <intermediate> <final>`. Charset
            // designation (`ESC ( B`, the ASCII reset a TUI emits after drawing
            // a box in the line-drawing set), `ESC % G` for UTF-8, `ESC # 8`,
            // and `ESC SP F`. Stripped as two bytes instead of three they leave
            // `(B`, `%G` or `#8` behind — and the position they leave it in is
            // the head of the line, which is the one place a stray pair costs a
            // cursor match and takes automatic approval down with it.
            //
            // The `is_ascii` test is what keeps every `end` in this match on a
            // character boundary, which is what keeps the slices below from
            // panicking. The other branches get that for free: a CSI final
            // byte, `BEL`, and `\` are all ASCII by definition, and this is the
            // one place the byte being stepped over is not.
            Some(b'(' | b')' | b'*' | b'+' | b'#' | b'%' | b' ')
                if bytes.get(index + 2).is_some_and(u8::is_ascii) =>
            {
                Some(index + 3)
            }
            Some(_) => Some(index + 1),
            None => None,
        };
        plain.push_str(&text[copied..index]);
        let Some(end) = end else {
            // Unterminated: there is no text after it to keep.
            return plain;
        };
        index = end;
        copied = end;
    }
    plain.push_str(&text[copied..]);
    plain
}

/// Where the OSC string opened at `start` ends, past its terminator.
///
/// `BEL` and `ESC \` both close one. xterm has accepted either for decades and
/// the agent CLIs use both, so a reader that knows only the standard one stops
/// at the wrong byte on half the terminals it meets.
fn operating_system_command_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start;
    while index < bytes.len() {
        match bytes[index] {
            0x07 => return Some(index + 1),
            0x1b if bytes.get(index + 1) == Some(&b'\\') => return Some(index + 2),
            _ => index += 1,
        }
    }
    None
}

pub(crate) fn relative_time(timestamp: u64) -> String {
    let seconds = now().saturating_sub(timestamp);
    match seconds {
        0..=59 => tr("たった今").into(),
        60..=3599 => tf!("{p0}分前", p0 = seconds / 60),
        3600..=86399 => tf!("{p0}時間前", p0 = seconds / 3600),
        _ => tf!("{p0}日前", p0 = seconds / 86400),
    }
}

/// How long until a moment in the future, said the way a person would.
///
/// Relative rather than a wall clock on purpose. A wall clock needs the local
/// offset, and this crate has neither a date library — a dependency is a paused
/// surface — nor a reason to reach `localtime_r` through `unsafe`. "In two
/// hours" is also the answer to the question actually being asked, which is
/// whether to keep working or to stop.
pub(crate) fn time_until(timestamp: u64) -> String {
    let seconds = timestamp.saturating_sub(now());
    match seconds {
        0 => tr("まもなく").into(),
        1..=3599 => tf!("あと {p0} 分", p0 = seconds.div_ceil(60)),
        3600..=86399 => tf!("あと {p0} 時間", p0 = seconds / 3600),
        _ => tf!("あと {p0} 日", p0 = seconds / 86400),
    }
}

pub(crate) fn open_in_external_editor(
    path: &Path,
    editor: crate::models::ExternalEditor,
) -> Result<()> {
    if !path.exists() {
        anyhow::bail!(
            "{}: path does not exist: {}",
            editor.label(),
            path.display()
        );
    }

    let mut command = Command::new(editor.command());
    command.arg(path);
    let direct_result = crate::exec::run_command_with_timeout(&mut command, Duration::from_secs(5));
    if let Ok(output) = direct_result {
        if output.status.success() {
            return Ok(());
        }
    }

    let app_name = match editor {
        crate::models::ExternalEditor::VsCode => Some("Visual Studio Code"),
        crate::models::ExternalEditor::Cursor => Some("Cursor"),
        crate::models::ExternalEditor::Zed => Some("Zed"),
        crate::models::ExternalEditor::Finder => None,
    };

    if let Some(app) = app_name {
        let mut fallback = Command::new("open");
        fallback.args(["-a", app]).arg(path);
        let fb_output =
            crate::exec::run_command_with_timeout(&mut fallback, Duration::from_secs(5))
                .map_err(|e| anyhow!("{}: {e}", editor.label()))?;
        if fb_output.status.success() {
            return Ok(());
        }
    }

    anyhow::bail!("{}: failed to launch editor", editor.label());
}
