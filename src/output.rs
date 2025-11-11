use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;

pub struct OutputEntry {
    pub callsign: String,
    pub name: String,
    pub suffix: String,
}

pub fn write_output_file(path: &str, entries: Vec<OutputEntry>) -> Result<()> {
    let mut file = File::create(path)
        .with_context(|| format!("Failed to create output file: {}", path))?;

    // Sort entries by callsign for consistent output
    let mut sorted_entries = entries;
    sorted_entries.sort_by(|a, b| a.callsign.cmp(&b.callsign));

    for entry in sorted_entries {
        writeln!(file, "{} {} {}", entry.callsign, entry.name, entry.suffix)
            .with_context(|| "Failed to write to output file")?;
    }

    Ok(())
}
