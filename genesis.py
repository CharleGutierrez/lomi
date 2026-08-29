import asyncio
import subprocess
import time
from google.antigravity import Agent, LocalAgentConfig
from google.antigravity.hooks import policy

def safe_git_workflow():
    print("Running cargo check...")
    check = subprocess.run(["cargo", "check"], capture_output=True, text=True)
    if check.returncode != 0:
        print("Compilation failed. Reverting changes...")
        subprocess.run(["git", "checkout", "--", "src/main.rs"])
        return

    status = subprocess.run(["git", "status", "--porcelain"], capture_output=True, text=True)
    if not status.stdout.strip():
        print("No changes to commit.")
        return

    timestamp = int(time.time())
    branch_name = f"genesis-patch-{timestamp}"
    
    print(f"Compilation passed. Moving changes to {branch_name}...")
    subprocess.run(["git", "checkout", "-b", branch_name])
    subprocess.run(["git", "commit", "-am", f"Auto-generated genesis patch {timestamp}"])
    subprocess.run(["git", "checkout", "master"])
    print("Done. Returned to master branch.")

async def main():
    config = LocalAgentConfig(
        system_instructions="You are the LOMI AI Genesis Agent. Read src/main.rs, find a minor inefficiency or typo, and apply a fix.",
        policies=[policy.allow_all()],
    )
    
    async with Agent(config) as agent:
        print("Agent is thinking...")
        response = await agent.chat("Please analyze src/main.rs, find a minor issue or typo, and fix it using your tools. Do not ask for user input. Finish when done.")
        print("\nAgent final response:")
        print(await response.text())
        
    safe_git_workflow()

if __name__ == "__main__":
    asyncio.run(main())
