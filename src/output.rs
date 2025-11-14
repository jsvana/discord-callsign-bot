use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;

pub struct OutputEntry {
    pub callsign: String,
    pub name: String,
    pub suffix: String,
    pub emoji_separator: String,
}

pub fn write_output_file(path: &str, entries: Vec<OutputEntry>, title: Option<&str>) -> Result<()> {
    let mut file =
        File::create(path).with_context(|| format!("Failed to create output file: {}", path))?;

    // Write title header if configured
    if let Some(title_text) = title {
        writeln!(file, "# TITLE: {}", title_text)
            .with_context(|| "Failed to write title to output file")?;
    }

    // Sort entries by callsign for consistent output
    let mut sorted_entries = entries;
    sorted_entries.sort_by(|a, b| a.callsign.cmp(&b.callsign));

    for entry in sorted_entries {
        writeln!(
            file,
            "{} {} {} {}",
            entry.callsign, entry.emoji_separator, entry.name, entry.suffix
        )
        .with_context(|| "Failed to write to output file")?;
    }

    Ok(())
}
