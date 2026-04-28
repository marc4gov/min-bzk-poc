---
name: Dynamic Mesh Actor Coordination Design
description: Design for a decentralized actor mesh using the ractor pattern for complex agent coordination.
type: architecture
date: 2026-04-28
---

# Dynamic Mesh Actor Coordination Design

## Overview
This design implements a decentralized "mesh" of actors to handle complex, multi-domain requests. Unlike a hierarchical orchestrator, this system utilizes a "Baton Pass" flow where actors delegate sub-tasks to peers, maintaining context through a shared envelope and deferred state.

## 1. Actor Roles & Responsibilities

### Entry Actor (Gateway)
- **Purpose**: Single entry point for external requests.
- **Responsibilities**: 
    - Request triage and initial analysis.
    - Identifying the first "Lead Expert".
    - Managing the final response delivery to the user.
    - Initiating cancellation signals.

### Registry Actor (Directory)
- **Purpose**: Service discovery and health monitoring.
- **Responsibilities**:
    - Mapping capabilities (e.g., "Rust", "React") to `ActorRef`s.
    - Monitoring expert health via heartbeats.
    - Providing `Unavailable` status for crashed actors.

### Expert Actors (Specialists)
- **Purpose**: Domain-specific logic execution.
- **Responsibilities**:
    - Solving domain-specific sub-problems.
    - Determining if delegation to another expert is required.
    - Managing internal deferred state for async delegation.

## 2. Messaging & Communication

### The Envelope Pattern
All inter-actor communication uses a wrapped `Envelope` to ensure traceability and return paths.

```rust
struct Envelope<M> {
    payload: M,
    context: SessionContext, 
    reply_to: ActorRef,      
    trace_id: Guid,          
    hop_count: u32,
}
```

### Communication Flow ("The Baton Pass")
1. **Triage**: `Entry` $\rightarrow$ `Registry` $\rightarrow$ `Lead Expert`.
2. **Delegation**: `Lead Expert` $\rightarrow$ `Registry` $\rightarrow$ `Peer Expert`.
3. **Context Propagation**: Every message carries the `SessionContext` and a `trace_id`.
4. **Return Path**: Results are sent back to the `reply_to` address, recreating the call stack logically.

### Constraints
- **Hop Limit**: Maximum of 5 hops to prevent circular delegation.
- **Non-Blocking**: Experts use a `PendingTasks` map to store state while waiting for peers.

## 3. State Management

### SessionContext
- Uses a **Reference-based Context**. 
- Carries a `SessionId` and immediate relevant state.
- Full history is retrieved from a shared `StorageActor` only when necessary.

### Deferred State
- When delegating, the actor saves the current execution state keyed by `trace_id`.
- Upon receiving a response with the matching `trace_id`, the actor resumes execution.

## 4. Stability & Error Recovery

### Timeouts & Retries
- Every delegated request starts a local timer.
- **Recovery Flow**: Timeout $\rightarrow$ Retry $\rightarrow$ Fallback Expert $\rightarrow$ Fail-fast.

### Health Monitoring
- `Registry Actor` tracks heartbeats.
- Immediate `ActorUnavailable` error is returned if a target is known to be crashed.

### Cancellation & Cleanup
- **Poison Pill**: `Entry Actor` broadcasts a `Cancel(trace_id)` signal.
- **Propagation**: Each expert propagates the signal to any actors they have delegated to.
- **Purge**: Actors remove the `trace_id` from their `PendingTasks` map.

## 5. Success Criteria
- **Correctness**: Complex requests are solved by the correct combination of experts.
- **Resilience**: System recovers gracefully from individual actor crashes.
- **Performance**: No deadlocks occur due to circular delegation; memory is cleaned up after cancellation.
