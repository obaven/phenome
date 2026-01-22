use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::io::Write;

pub(super) fn write_iterm2_image<W: Write>(
    stdout: &mut W,
    image: &[u8],
    area: ratatui::layout::Rect,
) -> Result<()> {
    let encoded = STANDARD.encode(image);
    write!(
        stdout,
        "\x1b]1337;File=inline=1;width={};height={};preserveAspectRatio=1:",
        area.width, area.height
    )?;
    stdout.write_all(encoded.as_bytes())?;
    write!(stdout, "\x07")?;
    Ok(())
}
