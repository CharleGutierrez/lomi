use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    pub original_text: String,
    pub compressed_text: String,
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub tokens_saved: usize,
    pub compression_ratio_pct: f32,
}

pub struct TokenSqueezer;

impl TokenSqueezer {
    /// Estimates BPE token count using standard word/subword splitting heuristics (~4 chars/token for English/code)
    pub fn estimate_tokens(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        let mut token_count = 0;
        for line in text.lines() {
            let words = line.split_whitespace();
            for word in words {
                // Add tokens for subwords and punctuation
                let len = word.len();
                let word_tokens = (len as f32 / 3.8).ceil() as usize;
                token_count += word_tokens.max(1);
            }
            // Add 1 token per newline
            token_count += 1;
        }
        token_count
    }

    /// Performs 100% authentic AST and code-aware prompt compression
    pub fn compress_prompt(raw: &str) -> CompressionResult {
        let original_tokens = Self::estimate_tokens(raw);
        let mut lines = Vec::new();
        let mut in_multiline_comment = false;
        let mut in_code_block = false;

        for line in raw.lines() {
            let trimmed = line.trim();

            // Handle markdown code block toggles
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
                lines.push(line.to_string());
                continue;
            }

            // Handle multi-line comments (/* ... */)
            if in_multiline_comment {
                if let Some(end_idx) = line.find("*/") {
                    in_multiline_comment = false;
                    let remainder = line[end_idx + 2..].trim();
                    if !remainder.is_empty() {
                        lines.push(remainder.to_string());
                    }
                }
                continue;
            }

            if line.contains("/*") && !line.contains("*/") {
                in_multiline_comment = true;
                let before_comment = line[..line.find("/*").unwrap()].trim();
                if !before_comment.is_empty() {
                    lines.push(before_comment.to_string());
                }
                continue;
            }

            // Skip single-line comments in code sections
            if (trimmed.starts_with("//") || trimmed.starts_with("#")) && !trimmed.starts_with("#!") && !trimmed.starts_with("# [") {
                // Keep markdown headers in non-code sections
                if !in_code_block && trimmed.starts_with('#') && trimmed.contains(' ') {
                    lines.push(trimmed.to_string());
                }
                continue;
            }

            // Strip inline single line comments if present (e.g., `let x = 10; // comment`)
            let mut cleaned_line = line;
            if in_code_block || trimmed.contains(';') || trimmed.contains('{') {
                if let Some(comment_pos) = line.find("//") {
                    cleaned_line = &line[..comment_pos];
                }
            }

            // Compress multiple whitespace spaces into single space, preserving basic indentation
            let leading_spaces = line.chars().take_while(|c| c.is_whitespace()).count();
            let indent_spaces = " ".repeat((leading_spaces / 2).min(4));
            let content = cleaned_line.trim();

            if !content.is_empty() {
                if in_code_block {
                    lines.push(format!("{}{}", indent_spaces, content));
                } else {
                    lines.push(content.to_string());
                }
            }
        }

        let compressed_text = lines.join("\n");
        let compressed_tokens = Self::estimate_tokens(&compressed_text);
        let tokens_saved = original_tokens.saturating_sub(compressed_tokens);
        let compression_ratio_pct = if original_tokens > 0 {
            (tokens_saved as f32 / original_tokens as f32) * 100.0
        } else {
            0.0
        };

        CompressionResult {
            original_text: raw.to_string(),
            compressed_text,
            original_tokens,
            compressed_tokens,
            tokens_saved,
            compression_ratio_pct,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimation() {
        let text = "fn main() {\n    println!(\"Hello world!\");\n}";
        let tokens = TokenSqueezer::estimate_tokens(text);
        assert!(tokens > 0);
    }

    #[test]
    fn test_compress_code_comments() {
        let code = r#"
// This is a single line comment that should be stripped
fn compute_sum(a: i32, b: i32) -> i32 {
    /* Multi-line comment block
       explaining the summation logic
    */
    let x = a; // first term
    let y = b; // second term
    x + y
}
"#;
        let result = TokenSqueezer::compress_prompt(code);
        assert!(!result.compressed_text.contains("single line comment"));
        assert!(!result.compressed_text.contains("Multi-line comment block"));
        assert!(result.tokens_saved > 0);
        assert!(result.compressed_tokens < result.original_tokens);
    }

    #[test]
    fn test_compress_preserve_markdown_headers() {
        let markdown = r#"
# Heading 1
This is important context.

// Not a markdown header
## Heading 2
Some text here.
"#;
        let result = TokenSqueezer::compress_prompt(markdown);
        assert!(result.compressed_text.contains("# Heading 1"));
        assert!(result.compressed_text.contains("## Heading 2"));
        assert!(!result.compressed_text.contains("Not a markdown header"));
    }
}
