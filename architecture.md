# AxumCan Architecture & Code Review Context

## Purpose

AxumCan is a Rust-based CAN simulator used to generate controllable CAN traffic for vehicle software development and testing.

The project is currently a prototype and is being gradually redesigned into a more complete CAN/vehicle simulator.

This document describes:

1. the current implementation
2. the intended architecture
3. known limitations
4. intentional design decisions
5. the scope of the current code review

Please review the existing code before proposing architectural changes.

---

# Current Architecture

The project currently contains the following main responsibilities:

- `registry.rs`
  - signal registration and runtime signal lookup/update

- `can_sender.rs`
  - cyclic CAN transmission tasks

- `signals.rs`
  - signal-related data structures

- `message_utils.rs`
  - CAN/message-related utilities

- `axum_can.rs`
  - Axum/WebSocket integration

- `main.rs`
  - application initialization and wiring

The current implementation should be treated as a prototype rather than the final architecture.

---

# Intended Signal Model

The target design is similar to a simplified DBC model.

A signal should eventually contain:

- signal name
- CAN channel
- CAN ID
- cycle time
- start bit
- bit length
- current/default value

A complete DBC file should not be required.

Signals belonging to the same CAN message should be grouped together.

Conceptually:

```text
CAN Message
├── channel
├── CAN ID
├── cycle time
└── signals
    ├── Signal A
    │   ├── start bit
    │   ├── length
    │   └── value
    └── Signal B
        ├── start bit
        ├── length
        └── value
```

The message task should periodically encode its current signals into a CAN frame and transmit it.

---

# Signal Identity — Important Design Decision

For the current version:

`signal_name` MUST remain the logical signal key.

Do not redesign this as part of the current review.

If the same signal name exists on multiple CAN channels, all matching instances are intentionally treated as the same logical signal.

Example:

```text
CAN1 / DoorLockSta
CAN2 / DoorLockSta
```

Updating:

```text
DoorLockSta = LOCKED
```

should affect both.

The assumption is that duplicate signal names represent the same physical/logical vehicle signal mirrored or forwarded across CAN buses.

A future design may distinguish physical signal identity using something similar to:

```text
(channel, CAN ID, signal_name)
```

while maintaining a separate logical signal mapping.

That change is explicitly OUT OF SCOPE for this review.

---

# CAN Message Scheduling

The current design uses one Tokio task per CAN message.

Each message has its own cycle time.

Conceptually:

```text
Message A ---- 10 ms ----> CAN
Message B ---- 20 ms ----> CAN
Message C --- 100 ms ----> CAN
```

The task-per-message design is intentional for the current expected workload because it keeps scheduling simple.

Review its correctness and lifecycle, but do not replace it solely because a centralized scheduler could theoretically scale better.

A redesign should only be proposed if there is a concrete correctness, resource, or scalability reason.

---

# Runtime Signal Updates

The frontend communicates with the Rust backend using WebSockets.

The intended data flow is:

```text
Frontend
   |
   | update signal
   v
Signal State
   |
   | latest values
   v
CAN Message Task
   |
   | encode
   v
CAN Frame
```

CAN transmission should use the latest available signal state.

The simulator does not need to preserve every intermediate frontend update.

For this reason, state-oriented primitives such as Tokio `watch` may be appropriate.

Do not automatically recommend replacing `watch` with `mpsc` unless message/event ordering is actually required.

---

# Known Gaps in the Current Prototype

The following are already known and should not be presented as newly discovered architectural requirements.

## CAN Encoding

The current implementation does not yet implement proper configurable:

- start bit
- signal length
- bit-level CAN encoding

The existing simplified encoding is temporary.

Review the existing code for correctness, but assume proper bit-level encoding is planned.

## CAN ID / Cycle Time

CAN ID and cycle time are not yet represented by the intended final configuration model.

The target architecture should make these explicit configuration values.

## CAN Interface Wiring

The current SocketCAN/interface abstraction is incomplete.

Review whether interface/socket ownership and dependency wiring are correct.

## Task Lifecycle

CAN message tasks currently require better lifecycle ownership.

The final design should make shutdown/cancellation explicit and avoid orphan or duplicate tasks.

---

# Future Vehicle Simulation

This functionality is NOT implemented yet.

Eventually AxumCan should support stateful vehicle behavior.

Example:

```text
DoorLockReq = LOCK
        |
        v
Vehicle Behavior
        |
        v
DoorLockSta = LOCKED
```

This represents a simulated ECU reacting to a CAN command.

Signals should eventually be configurable to allow or reject automatic backend changes.

This is needed to test both:

### Normal behavior

```text
DoorLockReq = LOCK
        ->
DoorLockSta = LOCKED
```

### Failure behavior

```text
DoorLockReq = LOCK
        ->
DoorLockSta remains unchanged
        ->
client timeout/failure handling
```

Do NOT implement this feature during the current code review unless explicitly requested.

---

# Code Review Priorities

Review the current code strictly in the following order.

## 1. Correctness

Look for behavior that can produce incorrect CAN state or unexpected application behavior.

Pay particular attention to:

- signal state consistency
- message grouping
- duplicate signal behavior
- initialization
- runtime updates
- stale state

---

## 2. Concurrency

Review:

- shared mutable state
- Tokio synchronization primitives
- lock scope
- locks across `.await`
- races between WebSocket updates and CAN generation
- consistency when multiple signals are encoded into one frame
- unnecessary cloning

Do not flag `watch` merely because intermediate values can be skipped. The intended semantic is primarily latest-state propagation.

---

## 3. Tokio Task Lifecycle

Review:

- ownership of spawned tasks
- dropped `JoinHandle`s
- cancellation
- shutdown
- repeated initialization
- duplicate tasks
- task panic/error handling

Determine whether a CAN sender can survive after its owning component is removed or reinitialized.

---

## 4. CAN Scheduling

Review:

- timer accuracy
- cycle-time drift
- `Instant` usage
- Tokio interval behavior
- `tokio::select!`
- starvation
- missed ticks
- behavior when CAN generation takes longer than the configured cycle

Pay particular attention to whether signal updates can starve periodic CAN transmission.

---

## 5. WebSocket Behavior

Review:

- multiple clients
- disconnected clients
- slow clients
- broadcast lag
- malformed messages
- stale initial state
- backpressure

WebSocket health must not affect cyclic CAN generation.

---

## 6. CAN Encoding

The current encoder is known to be temporary.

Still identify:

- unsafe assumptions
- payload overflow
- signal-count limits
- truncation
- invalid values
- silent incorrect output

Do not spend time designing the complete future DBC encoder unless requested.

---

## 7. Error Handling

Look for:

- `unwrap()`
- `expect()`
- ignored `Result`
- swallowed errors
- insufficient error context
- tasks terminating silently

Distinguish between:

- initialization errors
- configuration errors
- recoverable runtime errors
- client/WebSocket errors
- fatal errors

---

## 8. Testing

Identify missing tests for the CURRENT behavior.

Pay particular attention to:

- duplicate logical signal names
- multiple CAN messages
- multiple channels
- runtime signal updates
- cyclic transmission
- repeated initialization
- WebSocket disconnect
- slow WebSocket clients
- broadcast lag
- CAN send failures
- task cancellation

Prefer deterministic Tokio time tests instead of real sleeps.

---

# Review Constraints

Do NOT:

- redesign `signal_name` identity
- rewrite the entire project
- add abstractions only for cleanliness
- introduce a complex scheduler without evidence it is needed
- implement the future vehicle behavior engine
- treat planned functionality as if it already exists
- optimize for workloads the project does not currently need

Prefer small, explainable improvements.

---

# Expected Review Output

For every finding provide:

**Severity:** Critical / High / Medium / Low

**Location:** file + function/type

**Problem:** what is wrong

**Failure Scenario:** a concrete example of how it can fail

**Recommendation:** the smallest reasonable fix

**Priority:** fix now / fix before next feature / future improvement

Separate the final review into:

1. Correctness bugs
2. Concurrency / Tokio issues
3. Task lifecycle
4. CAN scheduling
5. WebSocket issues
6. CAN/data-model issues
7. Error handling
8. Testing gaps
9. Maintainability
10. Things that are currently acceptable and should NOT be redesigned

Do not generate code changes until the review is complete.
