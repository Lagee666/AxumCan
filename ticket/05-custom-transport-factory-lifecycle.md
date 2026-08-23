# 05: Custom transport factory and lifecycle

**What to build:** Library users can supply a custom asynchronous transport implementation that is created per CAN Channel and participates in startup, sending, shutdown, and error reporting.

**Blocked by:** 01: Explicit CAN Model with Print-mode transmission.

**Status:** ready-for-agent

- [ ] A global Custom transport mode accepts a user-provided factory.
- [ ] The factory receives each CAN Channel name and creates one transport for that Channel.
- [ ] Transport lifecycle operations run during startup and graceful shutdown.
- [ ] `send` receives an immutable encoded CAN Frame.
- [ ] Transport errors are returned and logged with transport, Channel, Message, CAN ID, and error context.
- [ ] A failed send does not terminate the periodic Message task; the next Cycle Time attempts delivery again.
- [ ] Factory failures and lifecycle failures prevent or correctly terminate initialization according to their phase.
- [ ] Fake transport tests verify frame delivery, Channel registration, lifecycle ordering, and send-error behavior.
