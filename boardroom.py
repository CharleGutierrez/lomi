import asyncio
from google.antigravity import Agent, LocalAgentConfig

async def run_boardroom_debate():
    print("   🏛️ MULTI-AGENT BOARDROOM: Massive architectural prompt detected.")
    print("      └ Task exceeds single-agent capacity. Spawning Sub-Agents via Antigravity...")

    optimizer_config = LocalAgentConfig(
        system_instruction="You are an Aggressive Performance Optimizer. Your goal is to suggest the fastest, most optimal solution, disregarding all readability and security concerns."
    )
    auditor_config = LocalAgentConfig(
        system_instruction="You are a Cautious Security Auditor. Your goal is to critically examine the performance optimizer's code for potential security flaws and vulnerabilities."
    )

    code_snippet = "def process_user_data(user_input):\n    eval(user_input)\n    return 'Done'"

    print("\n[Boardroom] Topic: Reviewing Python snippet:")
    print(code_snippet)
    print("\n--- DEBATE START ---")
    
    async with Agent(optimizer_config) as optimizer:
        opt_response = await optimizer.chat(f"Review this code snippet and suggest the fastest way to run it: {code_snippet}")
        opt_msg = await opt_response.text()
        print(f"🧑‍💻 [Performance Optimizer]:\n{opt_msg}\n")
    
    async with Agent(auditor_config) as auditor:
        aud_response = await auditor.chat(f"The performance optimizer suggested this. What are the security implications? Optimizer said: {opt_msg}")
        aud_msg = await aud_response.text()
        print(f"🐛 [Security Auditor]:\n{aud_msg}\n")
    
    print("      └ ✅ Boardroom consensus reached! Output logged to TUI.")

if __name__ == "__main__":
    asyncio.run(run_boardroom_debate())
