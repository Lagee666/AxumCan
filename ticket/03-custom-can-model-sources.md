# 03: Custom CAN Model sources

**What to build:** Library users can supply an asynchronous Rust model source, including their own DBC loader, and receive the same simulator behavior as the compatibility JSON source.

**Blocked by:** 01: Explicit CAN Model with Print-mode transmission.

**Status:** ready-for-agent

- [x] A public asynchronous `CanModelSource` extension point is available to library users.
- [x] A custom source can return the canonical CAN Model without depending on JSON or SocketCAN types.
- [x] The simulator loads, validates, and runs a custom source during startup.
- [x] Source failures prevent the simulator from starting and include useful context.
- [x] Equivalent legacy JSON and explicit/custom-source models produce equivalent canonical frames.
- [x] No built-in DBC parser is required; a user-provided DBC source is supported through the public seam.
