use regex::Regex;

#[derive(Debug, Clone)]
pub struct MemberInfo {
    pub callsign: String,
    pub name: String,
}

pub struct CallsignParser {
    // Matches amateur radio callsigns
    // Format: [prefix(1-2 chars)][digit][suffix(1-3 chars)]
    // Examples: W6JSV, KI7QCF, N0CALL, etc.
    callsign_regex: Regex,
}

impl CallsignParser {
    pub fn new() -> Self {
        // Pattern explanation:
        // \b - word boundary
        // [A-Z0-9]{1,2} - 1-2 character prefix (can be letters or numbers)
        // [0-9] - single digit
        // [A-Z]{1,3} - 1-3 letter suffix
        // \b - word boundary
        let callsign_regex = Regex::new(r"\b([A-Z0-9]{1,2}[0-9][A-Z]{1,3})\b")
            .expect("Failed to compile callsign regex");

        Self { callsign_regex }
    }

    /// Parse a Discord member's display name to extract callsign and name
    /// Handles formats like:
    /// - "W6JSV - Jay" -> callsign: W6JSV, name: Jay
    /// - "Forrest KI7QCF" -> callsign: KI7QCF, name: Forrest
    /// - "Jay (W6JSV)" -> callsign: W6JSV, name: Jay
    pub fn parse(&self, display_name: &str) -> Option<MemberInfo> {
        // Find the callsign in the display name
        let callsign_match = self.callsign_regex.find(display_name)?;
        let callsign = callsign_match.as_str().to_string();

        // Extract the name by removing the callsign and cleaning up
        let mut name = display_name.to_string();

        // Remove the callsign
        name = name.replace(&callsign, "");

        // Remove common separators and punctuation
        name = name
            .replace(" - ", " ")
            .replace(" -", "")
            .replace("- ", "")
            .replace("(", "")
            .replace(")", "")
            .trim()
            .to_string();

        // If name is empty, use the callsign as the name
        if name.is_empty() {
            name = callsign.clone();
        }

        Some(MemberInfo { callsign, name })
    }

    /// Validate if a string looks like a callsign
    #[allow(dead_code)]
    pub fn is_callsign(&self, text: &str) -> bool {
        self.callsign_regex.is_match(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_callsign_dash_name() {
        let parser = CallsignParser::new();
        let result = parser.parse("W6JSV - Jay").unwrap();
        assert_eq!(result.callsign, "W6JSV");
        assert_eq!(result.name, "Jay");
    }

    #[test]
    fn test_parse_name_callsign() {
        let parser = CallsignParser::new();
        let result = parser.parse("Forrest KI7QCF").unwrap();
        assert_eq!(result.callsign, "KI7QCF");
        assert_eq!(result.name, "Forrest");
    }

    #[test]
    fn test_parse_name_parens_callsign() {
        let parser = CallsignParser::new();
        let result = parser.parse("Jay (W6JSV)").unwrap();
        assert_eq!(result.callsign, "W6JSV");
        assert_eq!(result.name, "Jay");
    }

    #[test]
    fn test_parse_callsign_only() {
        let parser = CallsignParser::new();
        let result = parser.parse("W6JSV").unwrap();
        assert_eq!(result.callsign, "W6JSV");
        assert_eq!(result.name, "W6JSV");
    }

    #[test]
    fn test_is_callsign() {
        let parser = CallsignParser::new();
        assert!(parser.is_callsign("W6JSV"));
        assert!(parser.is_callsign("KI7QCF"));
        assert!(parser.is_callsign("N0CALL"));
        assert!(!parser.is_callsign("notacallsign"));
        assert!(!parser.is_callsign("123456"));
    }
}
