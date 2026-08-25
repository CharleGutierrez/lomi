# lomi — plan

> Editable **intent** file: ideas + plans — what we *mean to do*.
> This is NOT the event log. `events.jsonl` -> `summary.md` records what
> *happened*; this file records what we *intend*. The AI reads it at
> session start and edits it directly (like `PROJECT_MAP.md`): add ideas
> and plans, check items off, move done work down to Shipped. Plans are
> never logged as events.

## Ideas
_Loose thoughts, not yet committed to._

## Active plans
_What we're working toward now. Use `- [ ]` / `- [x]` checklists._

**Windows Features:**
- [ ] eBPF for Windows Integration (zero-overhead packet inspection)
- [ ] Native GUI with Tauri or Slint (replacing terminal/mock system tray)
- [ ] Local Desktop RAG (Windows Search / Everything IPC hook)
- [ ] Dynamic Core Parking & Power Plan AI (via PPM API)
- [ ] ONNX / DirectML Execution Engine (running local inference on NPU/GPU)

**Linux Features:**
- [ ] eBPF / XDP Zero-Copy Proxy Acceleration (microsecond routing)
- [ ] Firecracker MicroVM Sandboxing (fast KVM isolation for speculative code)
- [ ] Dynamic `cgroups v2` Omni-Tuning (CPU/memory throttling for AI tasks)
- [ ] `io_uring` Asynchronous Core (zero context switching for AI proxy streams)
- [ ] Wayland Layer-Shell Global Spotlight (Quake-style command palette)
- [ ] System-Wide D-Bus & journalctl RAG (local system context for the LLM)

**Deep Kernel Intelligence (Linux):**
- [ ] `mlock()` & `tmpfs` RAM-Disk (Zero-Latency Model pinning)
- [ ] Autonomous AppArmor / SELinux Policy Generation (AI-generated kernel security)
- [ ] GPU MIG / VFIO Passthrough (Hardware-isolated GPU slicing for Firecracker)
- [ ] `perf_event_open` CPU Cache-Miss Telemetry (Hardware-level performance counters)
- [ ] WireGuard Netlink AI Swarm (Kernel-level encrypted peer-to-peer compute)

**God-Tier HPC (Linux):**
- [ ] eBPF `uprobe` Memory Hijacking (Direct C++ heap token extraction)
- [ ] DPDK Polling (100-Gigabit kernel bypass line-rate proxy)
- [ ] NUMA-Aware CPU & Memory Pinning (Multi-socket PCIe bridge optimization)
- [ ] `SCHED_FIFO` Hard Real-Time (PREEMPT_RT 1000Hz jitter-free execution)
- [ ] WebAssembly (Wasm) Edge UDFs (Hot-loaded Wasmtime proxy middleware)

**Omni-Orchestrator Integration:**
- [ ] Central AI Orchestrator (`omni_orchestrator.rs`) that intelligently triggers eBPF, DPDK, cgroups, and RTOS based on real-time payload analysis.

## Next
_Queued, but not started._

## Someday / maybe

## Shipped
_Move completed plans here so the top stays about the future._
