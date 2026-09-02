use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub model: String,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub estimated_cost_usd: f64,
    pub is_local_free_compute: bool,
    pub rate_limit_allowed: bool,
    pub current_rpm: u32,
    pub max_rpm: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClientWindow {
    pub request_timestamps: Vec<i64>,
}

pub struct RateLimiter;

impl RateLimiter {
    const STATE_FILE: &'static str = ".lomi_cache/rate_limiter_state.json";

    /// Calculates monetary API cost estimation ($ USD) based on model provider pricing tiers
    pub fn calculate_cost(model: &str, prompt_tokens: usize, completion_tokens: usize) -> (f64, bool) {
        let model_lower = model.to_lowercase();

        if model_lower.contains("qwen") || model_lower.contains("llama") || model_lower.contains("mistral") || model_lower.contains("ollama") {
            return (0.0000, true);
        }

        let (prompt_rate, completion_rate) = match model_lower.as_str() {
            "gpt-4o" | "gpt-4o-mini" => (0.0025 / 1000.0, 0.0100 / 1000.0),
            "gpt-4" | "gpt-4-32k" => (0.0300 / 1000.0, 0.0600 / 1000.0),
            "claude-3-5-sonnet" | "claude-3-opus" => (0.0030 / 1000.0, 0.0150 / 1000.0),
            _ => (0.0015 / 1000.0, 0.0020 / 1000.0), // Standard default API rate
        };

        let cost = (prompt_tokens as f64 * prompt_rate) + (completion_tokens as f64 * completion_rate);
        (cost, false)
    }

    /// Evaluates sliding-window token-bucket rate limit (Requests Per Minute)
    pub fn evaluate(client_id: &str, model: &str, prompt_tokens: usize, completion_tokens: usize, max_rpm: u32) -> CostEstimate {
        let now_sec = chrono::Utc::now().timestamp();
        let cutoff_sec = now_sec - 60;

        let mut state = Self::load_state();
        let client_entry = state.entry(client_id.to_string()).or_insert_with(|| ClientWindow { request_timestamps: Vec::new() });

        // Filter timestamps within last 60 seconds
        client_entry.request_timestamps.retain(|&ts| ts > cutoff_sec);

        let current_rpm = client_entry.request_timestamps.len() as u32;
        let rate_limit_allowed = current_rpm < max_rpm;

        if rate_limit_allowed {
            client_entry.request_timestamps.push(now_sec);
            Self::save_state(&state);
        }

        let (estimated_cost_usd, is_local_free_compute) = Self::calculate_cost(model, prompt_tokens, completion_tokens);

        CostEstimate {
            model: model.to_string(),
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            estimated_cost_usd,
            is_local_free_compute,
            rate_limit_allowed,
            current_rpm: if rate_limit_allowed { current_rpm + 1 } else { current_rpm },
            max_rpm,
        }
    }

    fn load_state() -> HashMap<String, ClientWindow> {
        if Path::new(Self::STATE_FILE).exists() {
            if let Ok(content) = fs::read_to_string(Self::STATE_FILE) {
                if let Ok(map) = serde_json::from_str::<HashMap<String, ClientWindow>>(&content) {
                    return map;
                }
            }
        }
        HashMap::new()
    }

    fn save_state(state: &HashMap<String, ClientWindow>) {
        let _ = fs::create_dir_all(".lomi_cache");
        if let Ok(serialized) = serde_json::to_string_pretty(state) {
            let _ = fs::write(Self::STATE_FILE, serialized);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculation_local_free() {
        let (cost, is_free) = RateLimiter::calculate_cost("qwen2.5-coder:1.5b", 1000, 500);
        assert_eq!(cost, 0.0);
        assert!(is_free);
    }

    #[test]
    fn test_cost_calculation_gpt4o() {
        let (cost, is_free) = RateLimiter::calculate_cost("gpt-4o", 1000, 500);
        assert!(cost > 0.0);
        assert!(!is_free);
    }

    #[test]
    fn test_rate_limiter_rpm() {
        let res = RateLimiter::evaluate("test_client", "llama3.2:3b", 100, 50, 60);
        assert!(res.rate_limit_allowed);
        assert!(res.current_rpm > 0);
    }
}
