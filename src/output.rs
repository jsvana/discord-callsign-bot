#[derive(Debug)]
pub struct OutputEntry {
    pub callsign: String,
    pub name: String,
    pub suffix: String,
    pub emoji_separator: String,
}

pub fn generate_output_content(entries: Vec<OutputEntry>, title: Option<&str>) -> String {
    let mut output = String::new();

    // Write title header if configured
    if let Some(title_text) = title {
        output.push_str(&format!("# TITLE: {}\n", title_text));
    }

    // Sort entries by callsign for consistent output
    let mut sorted_entries = entries;
    sorted_entries.sort_by(|a, b| a.callsign.cmp(&b.callsign));

    for entry in sorted_entries {
        output.push_str(&format!(
            "{} {} {} {}\n",
            entry.callsign, entry.emoji_separator, entry.name, entry.suffix
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_output_content_basic() {
        let entries = vec![
            OutputEntry {
                callsign: "W6JSV".to_string(),
                name: "Jay".to_string(),
                suffix: "".to_string(),
                emoji_separator: "📻".to_string(),
            },
        ];

        let result = generate_output_content(entries, None);
        assert_eq!(result, "W6JSV 📻 Jay \n");
    }

    #[test]
    fn test_generate_output_content_with_title() {
        let entries = vec![
            OutputEntry {
                callsign: "W6JSV".to_string(),
                name: "Jay".to_string(),
                suffix: "".to_string(),
                emoji_separator: "📻".to_string(),
            },
        ];

        let result = generate_output_content(entries, Some("Test Title"));
        assert!(result.starts_with("# TITLE: Test Title\n"));
    }

    #[test]
    fn test_generate_output_content_sorts_by_callsign() {
        let entries = vec![
            OutputEntry {
                callsign: "KI7QCF".to_string(),
                name: "Forrest".to_string(),
                suffix: "".to_string(),
                emoji_separator: "📻".to_string(),
            },
            OutputEntry {
                callsign: "AA1AA".to_string(),
                name: "Alpha".to_string(),
                suffix: "".to_string(),
                emoji_separator: "📻".to_string(),
            },
        ];

        let result = generate_output_content(entries, None);
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines[0].starts_with("AA1AA"));
        assert!(lines[1].starts_with("KI7QCF"));
    }
}
