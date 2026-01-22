use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::io::Write;

pub(super) fn write_kitty_image<W: Write>(
    stdout: &mut W,
    image: &[u8],
    area: ratatui::layout::Rect,
    image_id: u32,
    is_tmux: bool,
) -> Result<()> {
    let encoded = STANDARD.encode(image);
    let chunk_size = 4096;
    let total_chunks = (encoded.len() + chunk_size - 1) / chunk_size;
    for (index, chunk) in encoded.as_bytes().chunks(chunk_size).enumerate() {
        let more = if index + 1 < total_chunks { 1 } else { 0 };
        let payload = if index == 0 {
            format!(
                "\x1b_Gf=100,a=T,c={},r={},i={},m={};",
                area.width, area.height, image_id, more
            )
        } else {
            format!("\x1b_Gm={};", more)
        };

        if is_tmux {
            write!(stdout, "\x1bPtmux;\x1b")?;
            write!(stdout, "{}", payload.replace("\x1b", "\x1b\x1b"))?;
        } else {
            write!(stdout, "{}", payload)?;
        }

        if is_tmux {
            for byte in chunk {
                write!(stdout, "{}", *byte as char)?;
            }
            write!(stdout, "\x1b\x1b\\")?;
            write!(stdout, "\x1b\\")?;
        } else {
            stdout.write_all(chunk)?;
            write!(stdout, "\x1b\\")?;
        }
    }
    Ok(())
}
