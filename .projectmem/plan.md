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

## Next
_Queued, but not started._

## Someday / maybe

## Shipped
_Move completed plans here so the top stays about the future._
