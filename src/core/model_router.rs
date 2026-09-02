use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelTier {
    UltraFastDraft, // Low complexity / fast completion
    BalancedLocal,  // Medium complexity / balanced accuracy & speed
    DeepTechCloud,  // High complexity / heavy architectural reasoning
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub requested_model: String,
    pub selected_model: String,
    pub selected_endpoint: String,
    pub complexity_score: u32,
    pub tier: ModelTier,
    pub fallback_chain: Vec<String>,
    pub endpoint_healthy: bool,
    pub routing_reason: String,
}

pub struct ModelRouter;

impl ModelRouter {
    /// Computes prompt complexity score from 0 to 100 based on structural & semantic features
    pub fn evaluate_prompt_complexity(prompt: &str) -> u32 {
        if prompt.is_empty() {
            return 0;
        }

        let mut score: u32 = 0;
        let char_count = prompt.len();
        let line_count = prompt.lines().count();

        // 1. Length & line count heuristic (up to 30 pts)
        if char_count > 4000 || line_count > 150 {
            score += 30;
        } else if char_count > 1000 || line_count > 40 {
            score += 20;
        } else if char_count > 300 || line_count > 10 {
            score += 10;
        }


        // 2. Code block & structural density (up to 35 pts)
        let code_blocks = prompt.matches("```").count() / 2;
        score += (code_blocks as u32 * 15).min(35);

        // 3. Technical / Deep reasoning keyword detection (up to 35 pts)
        let lower = prompt.to_lowercase();
        let keywords = [
            "refactor", "architect", "ebpf", "kernel", "deadlock", "memory leak",
            "concurrency", "optimization", "benchmark", "assembly", "unsafe",
            "security audit", "vulnerability", "proof of concept", "formal verification",
        ];

        for kw in keywords {
            if lower.contains(kw) {
                score += 5;
            }
        }

        score.min(100)
    }

    /// Checks whether an HTTP/Ollama model endpoint is responsive with a quick 300ms probe
    pub fn check_endpoint_health(endpoint: &str) -> bool {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(300))
            .build();

        if let Ok(c) = client {
            if let Ok(res) = c.get(endpoint).send() {
                return res.status().is_success() || res.status().is_redirection();
            }
        }
        false
    }

    /// Selects optimal target model and endpoint with automatic fallback chain
    pub fn route_request(
        requested_model: &str,
        prompt: &str,
        ollama_base_url: Option<&str>,
    ) -> RoutingDecision {
        let complexity_score = Self::evaluate_prompt_complexity(prompt);
        let base_url = ollama_base_url.unwrap_or("http://127.0.0.1:11434");

        let tier = if complexity_score >= 70 {
            ModelTier::DeepTechCloud
        } else if complexity_score >= 35 {
            ModelTier::BalancedLocal
        } else {
            ModelTier::UltraFastDraft
        };

        let (default_target, endpoint) = match tier {
            ModelTier::UltraFastDraft => (
                "qwen2.5-coder:1.5b".to_string(),
                format!("{}/api/generate", base_url),
            ),
            ModelTier::BalancedLocal => (
                "llama3.2:3b".to_string(),
                format!("{}/api/generate", base_url),
            ),
            ModelTier::DeepTechCloud => (
                "gpt-4o".to_string(),
                "https://api.openai.com/v1/chat/completions".to_string(),
            ),
        };

        // Honor explicit user model request if provided, otherwise auto-route
        let primary_model = if requested_model.is_empty() || requested_model == "auto" {
            default_target.clone()
        } else {
            requested_model.to_string()
        };

        // Probe endpoint health
        let health_check_url = format!("{}/api/tags", base_url);
        let ollama_healthy = Self::check_endpoint_health(&health_check_url);

        let mut fallback_chain = vec![
            "qwen2.5-coder:1.5b".to_string(),
            "llama3.2:3b".to_string(),
            "lomi-heuristic-fallback".to_string(),
        ];

        let selected_model = if !ollama_healthy && primary_model != "gpt-4o" {
            "lomi-heuristic-fallback".to_string()
        } else {
            primary_model.clone()
        };

        fallback_chain.retain(|m| m != &selected_model);

        let routing_reason = format!(
            "Complexity score {}/100 maps to {:?}. Ollama health check: {}",
            complexity_score, tier, if ollama_healthy { "ONLINE" } else { "OFFLINE (Using Fallback)" }
        );

        RoutingDecision {
            requested_model: requested_model.to_string(),
            selected_model,
            selected_endpoint: endpoint,
            complexity_score,
            tier,
            fallback_chain,
            endpoint_healthy: ollama_healthy,
            routing_reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_scoring() {
        let simple = "Hello, how are you?";
        let simple_score = ModelRouter::evaluate_prompt_complexity(simple);
        assert!(simple_score < 35);

        let complex = r#"
```rust
pub unsafe fn hijack_kernel_memory(ptr: *mut u8) {
    // Perform low-level eBPF assembly optimization and security audit for memory leak
}
```
Refactor this code to eliminate concurrency deadlock and benchmark performance.
"#;
        let complex_score = ModelRouter::evaluate_prompt_complexity(complex);
        assert!(complex_score >= 35);
    }

    #[test]
    fn test_routing_decision_auto() {
        let decision = ModelRouter::route_request("auto", "fn main() {}", None);
        assert!(!decision.selected_model.is_empty());
        assert!(!decision.selected_endpoint.is_empty());
    }
}
