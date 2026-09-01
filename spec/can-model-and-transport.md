# CAN Model and Transport Extension

## Problem Statement

AxumCan currently treats a CAN Message as a message name plus a flat map of signal names and numeric values. CAN IDs and Cycle Times are inferred from message names, signal values are copied into byte positions without signal layout metadata, and the sender is coupled to a mock SocketCAN implementation that prints frames.

This prevents AxumCan from producing correctly encoded classic CAN Frames and from using a real Linux SocketCAN interface when one is available. It also ties the simulator to one JSON configuration shape, even though users may need to load a DBC file or construct a CAN Model through their own code.

## Solution

Introduce a public, format-independent CAN Model and a validated classic CAN encoder. A user-provided asynchronous model source supplies the CAN Model at startup. The model describes CAN Channels, CAN Messages, Cycle Times, and Signals with enough metadata to encode real CAN Frames.

The application selects one global Transport Mode during Rust initialization:

- `Print`: display every encoded frame, labelled with its CAN Channel.
- `SocketCan`: open each CAN Channel as a Linux SocketCAN interface and transmit frames.
- `Custom`: use a user-provided transport factory, once per CAN Channel.

Every CAN Message is sent to exactly one destination determined by its channel. Transport mode is immutable after startup. A transport receives an already-encoded project-owned `CanFrame`; it does not know about signal layout or scaling.

## User Stories

1. As a CAN simulator user, I want to describe a CAN Message with an explicit CAN ID, so that the generated frame uses a predictable identifier instead of a name-derived hash.
2. As a CAN simulator user, I want to define a Message Cycle Time explicitly, so that periodic transmission follows the vehicle model rather than a naming convention.
3. As a CAN simulator user, I want to define a Signal's start bit and bit length, so that multiple Signals occupy the intended bits in one CAN Frame.
4. As a CAN simulator user, I want to select little-endian or big-endian encoding, so that the model can represent real CAN signal layouts.
5. As a CAN simulator user, I want to mark a Signal as signed or unsigned, so that negative and non-negative values are encoded correctly.
6. As a CAN simulator user, I want to define a Signal factor and offset, so that dashboard and simulator values can be physical values while CAN Frames contain raw encoded values.
7. As a CAN simulator user, I want to define Signal limits, so that invalid values are rejected before they produce an incorrect frame.
8. As a CAN simulator user, I want to provide initial Signal Values, so that the first transmitted frame has deterministic contents.
9. As a Linux CAN user, I want AxumCan to send valid classic CAN Frames to interfaces such as `vcan0` or `can0`, so that I can observe them with standard SocketCAN tools.
10. As a developer without CAN access, I want Print mode to display the same CAN ID and bytes that SocketCAN would send, so that I can validate encoding without hardware or an enabled `vcan` interface.
11. As a library user, I want to implement a custom asynchronous transport, so that frames can be sent to a recorder, test harness, remote service, or another CAN implementation.
12. As a library user, I want to implement a custom model source, so that AxumCan can load a DBC file or another domain format without requiring JSON.
13. As a library user, I want JSON shorthand configuration to remain usable, so that existing test fixtures can migrate without changing the simulator core.
14. As a library user, I want the JSON compatibility source and custom sources to produce the same CAN Model, so that encoding and scheduling do not depend on the source format.
15. As a simulator user, I want all Channels to use one globally selected Transport Mode, so that startup behavior is predictable and consistent.
16. As a simulator user, I want the Channel name to identify the SocketCAN interface, so that `vcan1` in the model maps directly to the Linux `vcan1` interface.
17. As a simulator user, I want startup to fail when a required SocketCAN interface cannot be opened, so that a test cannot appear successful while silently printing frames instead.
18. As a simulator user, I want transport failures to be returned and logged with context, so that delivery problems can be diagnosed without crashing the simulator.
19. As a simulator user, I want a failed periodic send to be retried on the next Cycle Time, so that a transient transport failure does not terminate the CAN Message task.
20. As a simulator user, I want invalid Signal layouts to fail during startup, so that overlapping, out-of-bounds, or otherwise malformed Signals never reach the encoder.
21. As a simulator user, I want duplicate Logical Signal names across CAN Channels to remain linked, so that mirrored vehicle Signals can be updated together.
22. As a dashboard user, I want runtime Signal Value updates to be encoded into subsequent frames, so that the CAN output reflects the latest simulator state.
23. As a dashboard user, I want observed State Changes to be broadcast to connected clients, so that the Control View and Monitor remain consistent.
24. As an operator, I want the transport mode and CAN Model to be selected only during startup, so that the running simulator does not switch interfaces or frame destinations unexpectedly.

## Implementation Decisions

- Introduce a public canonical CAN Model containing Channels, Messages, and Signals. The core model must not depend on JSON, DBC, SocketCAN, Axum, or WebSocket types.
- A CAN Message contains a name, CAN Channel, CAN ID, standard/extended identifier flag, Cycle Time, and one or more Signals.
- A Signal contains a name, start bit, bit length, byte order, signedness, factor, offset, optional minimum and maximum, and initial Signal Value.
- Use DBC-compatible semantics for start-bit numbering, little-endian encoding, and big-endian encoding. Do not introduce a separate start-byte convention.
- Support classic CAN in the first implementation. A frame payload is at most eight bytes. CAN FD and 64-byte payloads are out of scope.
- Add a project-owned `CanFrame` with an identifier, extended-ID flag, and encoded data bytes. Frame construction must validate standard IDs, extended IDs, and the classic CAN payload limit.
- The encoder converts physical Signal Values to raw values using the configured factor and offset, validates signedness and configured limits, and writes bits into the frame according to the Signal layout.
- Reject invalid models before sender tasks start. Invalid cases include unsupported or invalid CAN IDs, zero or invalid Cycle Times, factor zero, invalid limits, overlapping Signals, Signals outside the eight-byte frame, and unsupported multiplexing.
- Define an asynchronous `CanTransport` extension point with startup, frame-send, and shutdown lifecycle operations. The send operation receives an immutable encoded `CanFrame` and returns a transport error.
- Define a `TransportFactory` extension point that receives a CAN Channel name and creates one transport for that Channel.
- Select a global immutable `TransportMode` during application construction. The supported modes are Print, SocketCAN, and Custom.
- Print mode receives every encoded frame and displays its Channel, CAN ID, extended-ID state, and bytes. It does not require a Linux CAN interface.
- SocketCAN mode creates one transport per configured Channel and treats the Channel name as the Linux interface name. It must fail initialization if any required interface cannot be opened.
- Custom mode invokes the user-provided factory once per configured Channel. The custom implementation owns how that Channel is delivered.
- A CAN Message has exactly one destination: its Channel. There is no per-message output field and no multi-transport fan-out.
- Transport mode cannot change after initialization. Model loading, validation, transport creation, and transport startup complete before cyclic Message tasks begin.
- Transport send failures are returned and logged with transport, Channel, Message, CAN ID, and error context. A failed send does not terminate the periodic task; the task attempts delivery on the next Cycle Time.
- Transport startup failure aborts initialization. There is no silent fallback from SocketCAN to Print mode.
- Transport shutdown is explicit during graceful application shutdown. Runtime configuration reload is not part of the first implementation.
- Add a `CanModelSource` extension point with asynchronous loading. Users may supply a JSON compatibility source, a DBC source, or a fully custom source.
- AxumCan does not provide a built-in DBC parser. A user may implement a DBC source using any parser or library and return the canonical CAN Model.
- Preserve the existing flat JSON test format only in a compatibility adapter. Translate it into the canonical model immediately; no runtime component should consume the legacy structure directly.
- Keep the existing Logical Signal rule: same-named Signals on different Channels intentionally share one logical identity and update together.
- Preserve the existing latest-state behavior for periodic senders. A sender should encode the latest Signal Values rather than queue every intermediate dashboard update.
- Keep dashboard/WebSocket protocol behavior aligned with the canonical model. A client update changes simulator state, and accepted state changes are broadcast as State Changes to connected clients.

## Testing Decisions

- Tests should verify externally observable model validation, encoded bytes, transport calls, startup failures, periodic delivery, and WebSocket state propagation. They should not assert private data structures or task implementation details.
- Use a fake `CanModelSource` to supply deterministic models at the highest application seam. Use a fake `CanTransport` or `TransportFactory` to capture frames and errors without requiring Linux CAN interfaces.
- Add canonical frame-construction tests for valid standard and extended IDs, empty and full eight-byte payloads, invalid identifiers, and payload overflow.
- Add encoder tests for one unsigned eight-bit little-endian Signal, a multi-byte little-endian Signal, multiple Signals in one Message, a byte-crossing Signal, physical scaling, signed values, and a big-endian Signal.
- Add model-validation tests for overlapping Signals, out-of-bounds Signals, invalid factor and limits, zero/invalid Cycle Times, invalid CAN IDs, and unsupported multiplexing.
- Add transport tests proving Print and SocketCAN/custom transports receive the same encoded `CanFrame`, that Channel names are passed to custom factories, that duplicate or missing destinations fail initialization, and that startup/shutdown lifecycle calls occur in the expected order.
- Add failure tests proving initialization aborts when a SocketCAN interface cannot be opened and periodic sending continues after a send error.
- Add scheduling tests using deterministic Tokio time where possible, verifying that each Message transmits according to its Cycle Time and uses the latest Signal Values.
- Add model-source tests proving the legacy JSON adapter and a custom source produce equivalent canonical models for equivalent input.
- Add integration tests around the application builder using fake sources and transports to verify validation precedes task startup and that all configured Channels are initialized.
- Add WebSocket tests for initial model delivery, client Signal updates, arbitration behavior, State Change broadcasts, multiple clients, malformed messages, and disconnected clients.
- Use the existing unit-test style in the message utility and registry modules as prior art, but expand coverage around the new public seams rather than testing internal Tokio task mechanics.

## Out of Scope

- CAN FD and payloads larger than eight bytes.
- Built-in DBC parsing.
- Runtime model reload or runtime Transport Mode changes.
- Per-message multi-transport fan-out.
- Per-message output fields separate from the CAN Channel.
- Signal multiplexing.
- Enumeration/value-table metadata and symbolic dashboard values.
- Full DBC feature parity, including every attribute and advanced DBC construct.
- Stateful vehicle behavior or simulated ECU reactions.
- Redesigning Logical Signal identity to include Channel, CAN ID, or Signal instance.
- Replacing the intentional latest-state propagation with an event queue.
- Automatic fallback from SocketCAN to Print mode.

## Further Notes

The first useful end-to-end milestone is a single explicit CAN Message on a Channel such as `vcan0`, with one or more correctly encoded Signals, observed through Print mode when SocketCAN is unavailable. The same encoded bytes must then be deliverable through a user-selected SocketCAN or custom transport without changing the CAN Model or encoder.

The current project’s mock sender, name-derived CAN IDs and Cycle Times, flat test JSON, and direct signal-to-byte copying should be treated as compatibility or prototype behavior during migration, not as the canonical domain model.
