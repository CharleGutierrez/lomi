import subprocess
import time
import json
import re

COMMANDS_TO_TEST = [
    "tune", 
    "optimize-pi", 
    "test-hardware", 
    "swarm", 
    "index", 
    "genesis",
    "experimental",
    "orchestrate"
]

# We won't test `install-daemon` and `serve-proxy` via blocking calls immediately to avoid hanging,
# but we can test them via timeout.

def check_for_fake_keywords(output: str) -> list:
    fake_keywords = ['simulate', 'dummy', 'mock', 'placeholder', 'unimplemented', 'todo']
    found = []
    lower_out = output.lower()
    for kw in fake_keywords:
        if kw in lower_out:
            found.append(kw)
    return found

def run_command(cmd_args, timeout=5):
    try:
        print(f"\n[Agent] Testing: cargo run -- {' '.join(cmd_args)}")
        result = subprocess.run(
            ["cargo", "run", "--"] + cmd_args,
            capture_output=True,
            text=True,
            timeout=timeout
        )
        return result.stdout + "\n" + result.stderr
    except subprocess.TimeoutExpired as e:
        if e.stdout:
            out = e.stdout.decode('utf-8') if isinstance(e.stdout, bytes) else e.stdout
            err = e.stderr.decode('utf-8') if isinstance(e.stderr, bytes) else e.stderr
            return out + "\n" + (err or "")
        return "TIMEOUT"
    except Exception as e:
        return str(e)

def main():
    print("=== LOMI Autonomous Management Agent ===")
    print("Testing LOMI features for authentic implementations vs mock data...\n")
    
    results = {}
    
    for cmd in COMMANDS_TO_TEST:
        output = run_command([cmd], timeout=10)
        
        fakes = check_for_fake_keywords(output)
        
        # Analyze output for signs of being real or fake
        is_fake = len(fakes) > 0
        
        # specific logic
        if cmd == "test-hardware":
            is_fake = True # We know it simulates
            
        results[cmd] = {
            "status": "Fake/Simulated" if is_fake else "Looks Real (or undetermined)",
            "fake_keywords_found": fakes,
            "output_snippet": output[:300].replace('\n', ' ') + "..." if len(output) > 300 else output.replace('\n', ' ')
        }
        
    print("\n\n=== AGENT REPORT ===")
    for cmd, info in results.items():
        print(f"Feature '{cmd}': {info['status']}")
        if info['fake_keywords_found']:
            print(f"  -> Keywords flagged: {', '.join(info['fake_keywords_found'])}")
        print(f"  -> Snippet: {info['output_snippet'][:150]}")

if __name__ == "__main__":
    main()
