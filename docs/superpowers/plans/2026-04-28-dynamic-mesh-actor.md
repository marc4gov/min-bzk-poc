# Dynamic Mesh Actor Coordination Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a decentralized actor mesh using `ractor` for complex, multi-hop agent coordination with deferred state and failure recovery.

**Architecture:** A "Baton Pass" flow using `Envelope` wrappers. An `EntryActor` triages requests, a `RegistryActor` handles discovery/health, and `ExpertActors` manage their own state and delegate peers using a `PendingTasks` map for non-blocking async coordination.

**Tech Stack:** Rust, `ractor` (Actor framework), `uuid` (for trace IDs), `tokio` (runtime).

---

### Task 1: Core Types and Messaging ✅ (Completed)
**Files:** `agent-service/src/mesh/types.rs`, `agent-service/src/mesh/mod.rs`
- [x] Define `SessionContext` and `Envelope`
- [x] Define Mesh Message Types (`RegistryMsg`, `MeshSignal`)
- [x] Set up the mesh module

---

### Task 2: The Registry Actor (Next)
**Files:** `agent-service/src/mesh/registry.rs`
- [ ] **Step 1: Implement Registry State**: Define `RegistryActor` with `capabilities` and `health_map`.
- [ ] **Step 2: Implement Health Cleanup**: Method to remove stale actors.
- [ ] **Step 3: Commit**.

---

### Task 3: The Expert Base and Deferred State
**Files:** `agent-service/src/mesh/expert.rs`
- [ ] **Step 1: Define `ExpertState` and `PendingTask`**: Mechanism to store state while waiting for a peer.
- [ ] **Step 2: Implement the "Baton Pass" Logic**: Helper for delegation and hop limits (max 5).
- [ ] **Step 3: Commit**.

---

### Task 4: Entry Actor and Result Aggregation
**Files:** `agent-service/src/mesh/entry.rs`
- [ ] **Step 1: Implement Triage Logic**: Find the first expert via the Registry.
- [ ] **Step 2: Implement Final Result Handling**: Return final result to the user.
- [ ] **Step 3: Implement Cancellation Broadcasting**: Send `MeshSignal::Cancel`.
- [ ] **Step 4: Commit**.

---

### Task 5: Concrete Expert Implementations
**Files:** `agent-service/src/mesh/experts/mod.rs`, `rust_expert.rs`, `frontend_expert.rs`
- [ ] **Step 1: Implement `RustExpert`**: Process $\rightarrow$ Delegate $\rightarrow$ Save State $\rightarrow$ Resume.
- [ ] **Step 2: Implement `FrontendExpert`**: Similar logic for frontend domain.
- [ ] **Step 3: Commit**.

---

### Task 6: Stability Mechanisms (Timers & Poison Pills)
**Files:** `agent-service/src/mesh/expert.rs`, `agent-service/src/mesh/registry.rs`
- [ ] **Step 1: Implement Local Timeout Timers**: Trigger recovery when `PendingTasks` expire.
- [ ] **Step 2: Implement Poison Pill Propagation**: Handle `Cancel` signal and purge state.
- [ ] **Step 3: Commit**.

---

### Task 7: Integration Testing & Scenarios
**Files:** `agent-service/tests/mesh_tests.rs`
- [ ] **Step 1: Multi-Hop Success**: `Entry` $\rightarrow$ `Rust` $\rightarrow$ `Frontend` $\rightarrow$ `Rust` $\rightarrow$ `Entry`.
- [ ] **Step 2: Hop Limit Breach**: Verify `MaxHopsReached` error.
- [ ] **Step 3: Actor Failure Recovery**: Test timeout $\rightarrow$ fallback expert.
- [ ] **Step 4: Cancellation Propagation**: Verify state purge via `Cancel` signal.
- [ ] **Step 5: Commit**.
