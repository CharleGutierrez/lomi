import asyncio
from google.antigravity import Agent, LocalAgentConfig
from google.antigravity.hooks import policy

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

if __name__ == "__main__":
    asyncio.run(main())
