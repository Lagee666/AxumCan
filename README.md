# AxumCan

AxumCan is a generic, reusable simulation core library designed to bridge the gap between simple string-based mocking and complex, strongly-typed DBC hardware interactions. It provides a central Registry for managing signal states and a WebSocket-based synchronization layer for real-time dashboards.

## Core Concepts

The library is built around a generic Registry that manages:
- Signal values and arbitration states.
- WebSocket connections for UI synchronization.
- CAN Actor lifecycle management.

By leveraging Rust generics, the same core logic supports both "Zero-Config" development and "Production" hardware integration.

## Modes of Operation

### 1. Zero-Config Mode (Mocking)
Ideal for frontend development or quick prototyping. It uses plain strings for signals and messages. No DBC files or complex configuration required.

```rust
use axum_can::registry::Registry;

#[tokio::main]
async fn main() {
    // Uses default types: Registry<String, String, MockBuilder>
    let mut registry = Registry::default();
    registry.init().await.unwrap();
    
    // Update a signal - automatically creates a mock CAN actor
    registry.update("Engine_Speed".to_string(), 3000.0, false);
}
```

### 2. Production Mode (Real Hardware)
Integrate with generated DBC enums and real CAN interfaces by providing custom types and a hardware-specific Builder.

```rust
type AppRegistry = Registry<SignalLabel, MessageLabel, RealHardwareBuilder>;

// In your main loop
let mut registry = AppRegistry::default();
registry.init().await.unwrap();
```

## Key Traits

To implement custom hardware support, you only need to satisfy two traits:

- **SocketUtilsTrait**: Defines how to transform a map of signals into a raw CAN frame and how to send it.
- **CanBuilderTrait**: Defines how to initialize the socket and spawn the sending task.

## Why This Approach?

- **Clean Abstraction**: The business logic of state synchronization and arbitration is completely decoupled from the transport layer (Mock vs. Real SocketCAN).
- **Type Safety**: When using DBC enums, the compiler ensures you never send undefined signals.
- **Developer Experience**: "Zero-Config" defaults allow developers to start building the UI immediately without waiting for the backend hardware logic to be finalized.
- **Testability**: You can unit test your dashboard logic using the Mock mode and switch to Real mode for integration testing with zero changes to the Registry interaction code.
