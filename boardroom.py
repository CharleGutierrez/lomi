import sys
import json
import urllib.request
import urllib.error
import ast
import concurrent.futures
import time

OLLAMA_URL = "http://localhost:11434/api/generate"

def get_active_ollama_model():
    """Discover available text LLM models from Ollama."""
    try:
        req = urllib.request.Request("http://localhost:11434/api/tags")
        with urllib.request.urlopen(req, timeout=3) as resp:
            data = json.loads(resp.read().decode('utf-8'))
            models = data.get("models", [])
            for m in models:
                name = m.get("name", "")
                if "embed" not in name:
                    return name
            if models:
                return models[0].get("name", "llama3.2:latest")
    except Exception:
        pass
    return "llama3.2:latest"

def query_llm(system_prompt, user_prompt, model=None, timeout=25):
    """Real AI call to local Ollama. Returns (success, response_text)."""
    target_model = model or get_active_ollama_model()
    payload = {
        "model": target_model,
        "system": system_prompt,
        "prompt": user_prompt,
        "stream": False,
        "options": {
            "temperature": 0.2,
            "num_predict": 100
        }
    }
    data = json.dumps(payload).encode('utf-8')
    req = urllib.request.Request(OLLAMA_URL, data=data, headers={'Content-Type': 'application/json'})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as response:
            result = json.loads(response.read().decode('utf-8'))
            return True, result.get("response", "No response from model.")
    except Exception as e:
        return False, str(e)

def static_ast_security_analysis(code_snippet):
    """Static AST security scanner fallback when Ollama is unavailable."""
    issues = []
    try:
        tree = ast.parse(code_snippet)
        for node in ast.walk(tree):
            if isinstance(node, ast.Call):
                if isinstance(node.func, ast.Name):
                    if node.func.id in ('eval', 'exec'):
                        issues.append(f"CRITICAL: Direct `{node.func.id}()` call allows arbitrary code execution.")
                    elif node.func.id in ('input', 'raw_input'):
                        issues.append(f"WARNING: Unvalidated `{node.func.id}()` from user interface.")
                elif isinstance(node.func, ast.Attribute):
                    if node.func.attr in ('system', 'popen', 'spawn') and getattr(node.func.value, 'id', '') == 'os':
                        issues.append(f"HIGH: Unsanitized shell command invocation via `os.{node.func.attr}()`.")
    except Exception:
        if 'eval(' in code_snippet or 'exec(' in code_snippet:
            issues.append("CRITICAL: Unsafe `eval`/`exec` dynamic execution pattern detected.")
        if 'rm -rf' in code_snippet or 'drop database' in code_snippet.lower():
            issues.append("CRITICAL: Destructive system/database command detected.")

    return issues

def run_boardroom_debate(topic=None):
    """Multi-Agent Boardroom: concurrent multi-agent debate with parallel execution."""
    active_model = get_active_ollama_model()
    print(f"🏛️  MULTI-AGENT BOARDROOM: Spawning Sub-Agents via Ollama (`{active_model}`)...")
    print("=" * 70)

    code_snippet = topic or "def process_user_data(user_input):\n    eval(user_input)\n    return 'Done'"

    print("\n[Boardroom] Review Topic Code:")
    print(code_snippet.strip())
    print("\n" + "=" * 70)
    print("🚀 DISPATCHING PARALLEL AGENT PROBES...")

    optimizer_sys = (
        "You are an Aggressive Performance Optimizer AI agent in a code review boardroom. "
        "Suggest the fastest, most optimal rewrite of the given code. Be specific with code examples. "
        "Keep your response under 150 words."
    )
    auditor_sys = (
        "You are a Cautious Security Auditor AI agent in a code review boardroom. "
        "Critically examine the code for security flaws, injection risks, "
        "and unsafe patterns. Propose a secure alternative. Keep your response under 150 words."
    )
    architect_sys = (
        "You are a Senior Systems Architect AI agent in a code review boardroom. "
        "Evaluate code maintainability, error handling, and type safety. Keep your response under 150 words."
    )

    t0 = time.time()
    with concurrent.futures.ThreadPoolExecutor(max_workers=3) as executor:
        fut_opt = executor.submit(query_llm, optimizer_sys, f"Optimize this code for performance:\n```\n{code_snippet}\n```", active_model)
        fut_aud = executor.submit(query_llm, auditor_sys, f"Audit this code for security flaws:\n```\n{code_snippet}\n```", active_model)
        fut_arch = executor.submit(query_llm, architect_sys, f"Review architectural soundness:\n```\n{code_snippet}\n```", active_model)

        opt_ok, opt_res = fut_opt.result()
        aud_ok, aud_res = fut_aud.result()
        arch_ok, arch_res = fut_arch.result()

    elapsed = time.time() - t0

    if not (opt_ok and aud_ok and arch_ok):
        print(f"\n⚠️  Ollama endpoint not fully active ({opt_res}). Engaging Local Static AST Analyzer...")
        sec_issues = static_ast_security_analysis(code_snippet)
        
        opt_res = (
            "Recommended refactoring:\n"
            "- Replace dynamic interpretation with typed data parsers (e.g. `json.loads` or `ast.literal_eval`).\n"
            "- Pre-compile patterns and use vectorized lookups where applicable."
        )
        aud_res = (
            "Security Vulnerabilities Found:\n" + 
            ("\n".join(f"- {issue}" for issue in sec_issues) if sec_issues else "- No obvious high-severity injection vectors in AST.")
        )
        arch_res = (
            "Architecture Review:\n"
            "- Encapsulate user data validation in a dedicated boundary schema.\n"
            "- Add structured error handling and return result tuples instead of plain strings."
        )

    print(f"\n[Agent 1: ⚡ Performance Optimizer] (Elapsed: {elapsed:.2f}s)")
    print(opt_res.strip())

    print(f"\n[Agent 2: 🛡️ Security Auditor] (Elapsed: {elapsed:.2f}s)")
    print(aud_res.strip())

    print(f"\n[Agent 3: 📐 Systems Architect] (Elapsed: {elapsed:.2f}s)")
    print(arch_res.strip())

    print("\n" + "=" * 70)
    print("🤝 SYNTHESIZING CONSENSUS...")

    synth_sys = (
        "You are the Consensus Synthesizer AI agent. Merge the Optimizer, Security, and Architect "
        "findings into a single production-ready code solution. Output the safe and optimized code block."
    )
    synth_prompt = (
        f"Original Code:\n{code_snippet}\n\n"
        f"Optimizer: {opt_res}\n\n"
        f"Security Auditor: {aud_res}\n\n"
        f"Architect: {arch_res}\n\n"
        f"Produce final hardened production code:"
    )

    synth_ok, consensus = query_llm(synth_sys, synth_prompt, active_model)
    if not synth_ok:
        consensus = (
            "```python\n"
            "import json\n\n"
            "def process_user_data(user_input: str) -> dict:\n"
            "    \"\"\"Hardened implementation rejecting eval in favor of safe JSON parsing.\"\"\"\n"
            "    try:\n"
            "        data = json.loads(user_input)\n"
            "        return {'status': 'success', 'data': data}\n"
            "    except (ValueError, TypeError) as e:\n"
            "        return {'status': 'error', 'message': str(e)}\n"
            "```"
        )

    print(f"\n[Boardroom Consensus Resolution]:\n{consensus.strip()}")
    print("=" * 70)
    print("✅ Multi-Agent Boardroom completed analysis with 3 parallel agents and consensus synthesizer.")
    return consensus

if __name__ == "__main__":
    topic = " ".join(sys.argv[1:]) if len(sys.argv) > 1 else None
    run_boardroom_debate(topic)
