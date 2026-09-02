use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use crate::core::token_squeezer::TokenSqueezer;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRotationResult {
    pub original_message_count: usize,
    pub rotated_message_count: usize,
    pub original_token_count: usize,
    pub rotated_token_count: usize,
    pub archived_message_count: usize,
    pub rotated_messages: Vec<ChatMessage>,
    pub archive_file: String,
}

pub struct ContextRotator;

impl ContextRotator {
    /// Estimates total tokens across a list of chat messages
    pub fn estimate_total_tokens(messages: &[ChatMessage]) -> usize {
        messages
            .iter()
            .map(|m| TokenSqueezer::estimate_tokens(&m.content) + 4) // +4 tokens per message overhead
            .sum()
    }

    /// Performs dynamic context rotation and windowing to fit within max_token_budget
    pub fn rotate_context(messages: &[ChatMessage], max_token_budget: usize) -> ContextRotationResult {
        let original_message_count = messages.len();
        let original_token_count = Self::estimate_total_tokens(messages);

        if original_token_count <= max_token_budget || original_message_count <= 4 {
            return ContextRotationResult {
                original_message_count,
                rotated_message_count: original_message_count,
                original_token_count,
                rotated_token_count: original_token_count,
                archived_message_count: 0,
                rotated_messages: messages.to_vec(),
                archive_file: String::new(),
            };
        }

        // Separate system messages, middle turns, and recent turns
        let mut system_messages = Vec::new();
        let mut conversation_turns = Vec::new();

        for msg in messages {
            if msg.role == "system" {
                system_messages.push(msg.clone());
            } else {
                conversation_turns.push(msg.clone());
            }
        }

        // Keep last 4 conversation turns intact
        let keep_recent_count = 4.min(conversation_turns.len());
        let split_idx = conversation_turns.len().saturating_sub(keep_recent_count);

        let middle_turns = &conversation_turns[..split_idx];
        let recent_turns = &conversation_turns[split_idx..];

        // Archive middle turns to .lomi_cache/archived_context.jsonl
        let archive_path = ".lomi_cache/archived_context.jsonl";
        let _ = fs::create_dir_all(".lomi_cache");
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(archive_path) {
            for turn in middle_turns {
                if let Ok(serialized) = serde_json::to_string(turn) {
                    let _ = writeln!(file, "{}", serialized);
                }
            }
        }

        // Summarize middle turns into a single compact context memory message
        let summary_text = middle_turns
            .iter()
            .take(5)
            .map(|t| format!("[{}: {}]", t.role, t.content.chars().take(80).collect::<String>()))
            .collect::<Vec<String>>()
            .join(" | ");

        let context_summary_msg = ChatMessage {
            role: "system".to_string(),
            content: format!(
                "📜 [LOMI Context Memory Summary ({} archived turns)]: {}",
                middle_turns.len(),
                summary_text
            ),
        };

        let mut rotated_messages = Vec::new();
        rotated_messages.extend(system_messages);
        rotated_messages.push(context_summary_msg);
        rotated_messages.extend_from_slice(recent_turns);

        let rotated_token_count = Self::estimate_total_tokens(&rotated_messages);

        ContextRotationResult {
            original_message_count,
            rotated_message_count: rotated_messages.len(),
            original_token_count,
            rotated_token_count,
            archived_message_count: middle_turns.len(),
            rotated_messages,
            archive_file: archive_path.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;


    #[test]
    fn test_context_rotation_no_op_small_budget() {
        let msgs = vec![
            ChatMessage { role: "system".to_string(), content: "You are a helpful AI assistant.".to_string() },
            ChatMessage { role: "user".to_string(), content: "Hi!".to_string() },
        ];

        let res = ContextRotator::rotate_context(&msgs, 1000);
        assert_eq!(res.rotated_message_count, 2);
        assert_eq!(res.archived_message_count, 0);
    }

    #[test]
    fn test_context_rotation_archives_middle_turns() {
        let mut msgs = vec![
            ChatMessage { role: "system".to_string(), content: "System directive".to_string() },
        ];

        for i in 1..=10 {
            msgs.push(ChatMessage {
                role: if i % 2 == 0 { "assistant".to_string() } else { "user".to_string() },
                content: format!("Detailed turn message number {} containing lots of context words.", i),
            });
        }

        // Budget forced small so it must rotate
        let res = ContextRotator::rotate_context(&msgs, 50);
        assert!(res.archived_message_count > 0);
        assert!(res.rotated_message_count < res.original_message_count);
        assert!(Path::new(".lomi_cache/archived_context.jsonl").exists());
    }
}
