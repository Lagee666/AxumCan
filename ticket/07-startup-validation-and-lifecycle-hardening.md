# 07: Startup validation and sender lifecycle hardening

**What to build:** The simulator starts only from a valid model and transport configuration, and its periodic sender and WebSocket tasks shut down or recover without leaks, busy loops, or duplicate transmission.

**Blocked by:** 02: Complete classic CAN Signal encoding; 04: Linux SocketCAN transport mode; 05: Custom transport factory and lifecycle; 06: Dashboard Signal State Synchronization.

**Status:** ready-for-agent

- [x] Model validation completes before any cyclic sender task starts.
- [x] Transport initialization completes before any cyclic sender task starts.
- [x] Initialization failures leave no running sender tasks or partially active transports.
- [x] Repeated initialization does not leave duplicate or orphaned sender tasks.
- [x] Sender tasks exit cleanly when their owning simulator shuts down.
- [x] A closed state channel does not cause a busy loop.
- [x] Transport send failures are logged and periodic transmission continues.
- [x] WebSocket send/receive task failures do not stop unrelated CAN sender tasks.
- [x] Deterministic tests cover initialization failure, shutdown, repeated initialization, task cancellation, and send failure.
