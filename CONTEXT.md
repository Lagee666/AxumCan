# AxumCan Domain Context

AxumCan simulates controllable CAN traffic for vehicle software development and testing. This context defines the vocabulary for configured vehicle signals, their CAN messages, and the dashboard controls that change or observe them.

## CAN Model

**CAN Channel**:
A bus or interface on which CAN messages are transmitted, such as `vcan1` or `vcan2`.
_Avoid_: bus ID, signal channel

**CAN Message**:
A cyclic unit of CAN traffic transmitted on one CAN channel and containing one or more signals.
_Avoid_: signal, packet

**CAN Frame**:
The encoded, wire-level representation of a CAN message at one transmission instant.
_Avoid_: message payload, signal

**Signal**:
A named vehicle value carried by a CAN message, such as speed, mode, or switch state.
_Avoid_: field, variable, parameter

**Logical Signal**:
The vehicle-level identity represented by a signal name. Signals with the same name on multiple CAN channels intentionally refer to the same logical signal and are updated together.
_Avoid_: physical signal, signal instance

**Signal Value**:
The current numeric value of a logical signal.
_Avoid_: payload, state message

**Cycle Time**:
The interval at which a CAN message is transmitted repeatedly.
_Avoid_: timeout, refresh rate

## Simulation Behavior

**CAN Simulator**:
The system that maintains configured signal values and periodically emits CAN frames for their messages.
_Avoid_: CAN gateway, ECU

**Backend Control**:
Permission for simulator behavior to change a logical signal's value.
_Avoid_: ownership, lock, authorization

**Arbitration**:
The rule that determines whether backend-controlled changes are allowed for a logical signal.
_Avoid_: synchronization, conflict resolution

**Client Update**:
A value change requested manually by a dashboard user.
_Avoid_: backend update, event

**State Change**:
An observed change in a logical signal's value that can be broadcast to connected dashboard clients.
_Avoid_: client update, CAN frame

## Dashboard Views

**Dashboard**:
The browser interface used to change signal values and backend-control settings and to observe simulator state.
_Avoid_: frontend, monitor

**Control View**:
The dashboard view that presents configurable signals and allows users to issue client updates.
_Avoid_: signal page, editor

**Monitor**:
The dashboard view that displays a chronological stream of observed state changes.
_Avoid_: log, trace, event history
