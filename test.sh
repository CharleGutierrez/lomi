#!/bin/bash
cd "/home/cog/Pi Assisted Projects/lomi"
cargo build --release > build.log 2>&1
echo "BUILD DONE" > build.log.done
./target/release/lomi --help > help.log 2>&1
timeout 3 ./target/release/lomi serve-proxy --port 8080 > serve.log 2>&1
./target/release/lomi benchmark > bench.log 2>&1
timeout 3 ./target/release/lomi orchestrate > orch.log 2>&1
./target/release/lomi test-hardware > hw.log 2>&1
timeout 3 ./target/release/lomi swarm --mode host > swarm.log 2>&1
./target/release/lomi install-daemon > daemon.log 2>&1
timeout 5 ./target/release/lomi genesis > genesis.log 2>&1
cargo run -- experimental --feature gui > gui.log 2>&1
timeout 8 ./target/release/lomi serve-proxy --port 8080 > serve2.log 2>&1 &
sleep 2
curl -s -X POST http://127.0.0.1:8080/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{"model":"gpt-4","messages":[{"role":"user","content":"Hello"}]}' > curl.log 2>&1
wait
echo "ALL DONE" > all.log.done
