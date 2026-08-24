# 06: Dashboard Signal State Synchronization

**What to build:** Dashboard updates modify the canonical Signal state, subsequent CAN Frames use the latest values, and accepted State Changes reach every connected dashboard and the Monitor.

**Blocked by:** 01: Explicit CAN Model with Print-mode transmission; 02: Complete classic CAN Signal encoding.

**Status:** ready-for-agent

- [x] A dashboard Client Update changes the corresponding Logical Signal Value.
- [x] The next periodic CAN Frame uses the latest value without requiring an event queue.
- [x] Accepted updates produce State Change broadcasts.
- [x] Multiple connected dashboards receive the same State Change.
- [x] Backend Control and Arbitration continue to determine whether backend changes are accepted.
- [x] Unknown or malformed WebSocket messages do not terminate the connection or corrupt Signal state.
- [x] WebSocket tests cover initialization, updates, broadcasts, arbitration, multiple clients, and disconnects.
