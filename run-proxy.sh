#!/bin/bash

# ==========================================
# LOMI UPSTREAM CONFIGURATION
# ==========================================

# 1. Replace this with your actual API key (OpenAI, Groq, Anthropic, etc.)
# If using a local Ollama server, you can leave this as "ollama"
export UPSTREAM_API_KEY="sk-your-api-key-here"

# 2. Set the target API endpoint you want LOMI to forward requests to:
# OpenAI: https://api.openai.com/v1/chat/completions
# Groq:   https://api.groq.com/openai/v1/chat/completions
# Ollama: http://127.0.0.1:11434/v1/chat/completions
export UPSTREAM_BASE_URL="https://api.openai.com/v1/chat/completions"

echo "=========================================="
echo "🚀 Starting LOMI API Gateway Proxy..."
echo "🌐 Upstream Target: $UPSTREAM_BASE_URL"
echo "=========================================="

# Build and run the LOMI proxy
cargo run -- serve-proxy
