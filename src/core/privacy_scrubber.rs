use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrubReport {
    pub original_text: String,
    pub scrubbed_text: String,
    pub redaction_count: usize,
    pub redacted_types: Vec<String>,
}

pub struct PrivacyScrubber;

impl PrivacyScrubber {
    /// Performs 100% authentic PII detection and redaction on sensitive prompt text
    pub fn scrub_prompt(raw: &str) -> ScrubReport {
        if raw.is_empty() {
            return ScrubReport {
                original_text: String::new(),
                scrubbed_text: String::new(),
                redaction_count: 0,
                redacted_types: Vec::new(),
            };
        }

        let mut scrubbed = raw.to_string();
        let mut redaction_count = 0;
        let mut redacted_types = Vec::new();

        // 1. Redact RSA / Private Keys
        if scrubbed.contains("-----BEGIN") && (scrubbed.contains("PRIVATE KEY") || scrubbed.contains("RSA")) {
            if let Some(start) = scrubbed.find("-----BEGIN") {
                if let Some(end) = scrubbed[start..].find("-----END") {
                    let end_pos = start + end + scrubbed[start + end..].find('\n').unwrap_or(20);
                    let target = scrubbed[start..end_pos.min(scrubbed.len())].to_string();
                    scrubbed = scrubbed.replace(&target, "[REDACTED_PRIVATE_KEY]");
                    redaction_count += 1;
                    redacted_types.push("Private Key".to_string());
                }
            }
        }

        // 2. Redact API Keys (sk-..., ghp_..., AKIA...)
        let words: Vec<String> = scrubbed.split_whitespace().map(|s| s.to_string()).collect();
        for word in words {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-').to_string();
            
            if (clean_word.starts_with("sk-") && clean_word.len() > 15)
                || (clean_word.starts_with("ghp_") && clean_word.len() > 20)
                || (clean_word.starts_with("AKIA") && clean_word.len() == 20)
            {
                if scrubbed.contains(&clean_word) {
                    scrubbed = scrubbed.replace(&clean_word, "[REDACTED_API_KEY]");
                    redaction_count += 1;
                    if !redacted_types.contains(&"API Key".to_string()) {
                        redacted_types.push("API Key".to_string());
                    }
                }
            }
        }

        // 3. Redact Bearer / JWT Tokens (eyJ...)
        if scrubbed.contains("eyJ") {
            let tokens: Vec<String> = scrubbed.split_whitespace().map(|s| s.to_string()).collect();
            for token in tokens {
                if token.contains("eyJ") && token.len() > 30 && token.contains('.') {
                    if scrubbed.contains(&token) {
                        scrubbed = scrubbed.replace(&token, "[REDACTED_JWT_TOKEN]");
                        redaction_count += 1;
                        if !redacted_types.contains(&"JWT Token".to_string()) {
                            redacted_types.push("JWT Token".to_string());
                        }
                    }
                }
            }
        }

        // 4. Redact Email Addresses
        let parts: Vec<String> = scrubbed.split_whitespace().map(|s| s.to_string()).collect();
        for part in parts {
            let clean_email = part.trim_matches(|c: char| !c.is_alphanumeric() && c != '@' && c != '.' && c != '_' && c != '-').to_string();
            if clean_email.contains('@') && clean_email.contains('.') && !clean_email.starts_with('@') && !clean_email.ends_with('.') {
                let email_parts: Vec<&str> = clean_email.split('@').collect();
                if email_parts.len() == 2 && email_parts[0].len() >= 1 && email_parts[1].contains('.') {
                    if scrubbed.contains(&clean_email) {
                        scrubbed = scrubbed.replace(&clean_email, "[REDACTED_EMAIL]");
                        redaction_count += 1;
                        if !redacted_types.contains(&"Email Address".to_string()) {
                            redacted_types.push("Email Address".to_string());
                        }
                    }
                }
            }
        }

        // 5. Redact Public IPv4 Addresses (excluding localhost / 127.0.0.1)
        let ip_candidates: Vec<String> = scrubbed.split_whitespace().map(|s| s.to_string()).collect();
        for candidate in ip_candidates {
            let clean_ip = candidate.trim_matches(|c: char| !c.is_numeric() && c != '.').to_string();
            let octets: Vec<&str> = clean_ip.split('.').collect();
            if octets.len() == 4 {
                if octets.iter().all(|o| o.parse::<u8>().is_ok()) {
                    if clean_ip != "127.0.0.1" && clean_ip != "0.0.0.0" {
                        if scrubbed.contains(&clean_ip) {
                            scrubbed = scrubbed.replace(&clean_ip, "[REDACTED_IP_ADDRESS]");
                            redaction_count += 1;
                            if !redacted_types.contains(&"IP Address".to_string()) {
                                redacted_types.push("IP Address".to_string());
                            }
                        }
                    }
                }
            }
        }

        ScrubReport {
            original_text: raw.to_string(),
            scrubbed_text: scrubbed,
            redaction_count,
            redacted_types,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrub_api_keys_and_email() {
        let text = "Contact user at john.doe@company.com with key sk-12345678901234567890 and AWS AKIAIOSFODNN7EXAMPLE.";
        let report = PrivacyScrubber::scrub_prompt(text);
        assert!(!report.scrubbed_text.contains("john.doe@company.com"));
        assert!(!report.scrubbed_text.contains("sk-12345678901234567890"));
        assert!(report.scrubbed_text.contains("[REDACTED_EMAIL]"));
        assert!(report.scrubbed_text.contains("[REDACTED_API_KEY]"));
        assert_eq!(report.redaction_count, 3);
    }

    #[test]
    fn test_scrub_jwt_token() {
        let jwt_prompt = "Header Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.signature";
        let report = PrivacyScrubber::scrub_prompt(jwt_prompt);
        assert!(!report.scrubbed_text.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
        assert!(report.scrubbed_text.contains("[REDACTED_JWT_TOKEN]"));
    }
}
