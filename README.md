# AxumCan - CAN Simulator Dashboard

AxumCan is a real-time CAN signal simulator featuring a Rust backend and an interactive TypeScript/Tailwind CSS frontend dashboard. It supports full-duplex communication via WebSockets, allowing you to monitor and manipulate CAN signals dynamically with built-in control arbitration.

---

## Tech Stack

- **Backend:** Rust, Axum (Web server), Tokio (Async runtime), SocketCAN (via `socketcan` crate, mocked by default).
- **Frontend:** Single Page Application using TypeScript, Vite, and Tailwind CSS 4.0.
- **Protocol:** JSON-based WebSockets for real-time signaling.

---

## Getting Started

### 1. Build and Run the Complete Application (Recommended)

The Axum backend is configured to serve the frontend's built production files directly from `dashboard/dist`.

```bash
# 1. Build the frontend production assets
cd dashboard
npm install
npm run build
cd ..

# 2. Start the Rust server
cargo run
```

The application will start, serving the static dashboard and WebSocket handler on:
**http://localhost:8028**

### 2. Frontend Development Server (Hot Reloading)

If you are developing the frontend, you can run Vite's dev server:

```bash
cd dashboard
npm run dev
```

*Note: The frontend dynamically resolves the WebSocket connection to use the correct hostname and port, meaning it will connect back to the Axum backend automatically.*

---

## Configuration (can_signal.json)

Signals are loaded dynamically from can_signal.json at startup. The structure defines the CAN channels, messages, and default signal values:

```json
{
  "vcan1": {
    "test1": {
      "Speed": 0
    }
  },
  "vcan2": {
    "test2": {
      "Mode": 3
    }
  }
}
```

---

## Key Features

### 1. Control Arbitration (Backend vs. Frontend)
Each signal card on the dashboard contains a **Backend Control** toggle.
* **Enabled (default):** The backend can overwrite and update the signal's value dynamically.
* **Disabled:** The user gains exclusive control over the signal. Any updates pushed from the backend will be ignored for this signal until backend control is re-enabled.
* **Global Control:** A master toggle in the header allows enabling/disabling backend control for all signals simultaneously.

### 2. Real-Time Monitor Log
Switch to the **Monitor** view via the header navigation to see a rolling chronological log of all signal updates broadcast by the backend.

---

## WebSocket Protocol

Messages exchanged between Client (C) and Server (S) use a `type` discriminator and `camelCase` naming:

| Message Type | Direction | Payload | Description |
| :--- | :--- | :--- | :--- |
| `init` | S -> C | `signals: Signals` | Sends the full initial signal map to the client. |
| `clientUpdate` | C -> S | `signal: string, value: f64` | User changed a value manually on the dashboard. |
| `setArbitration`| C -> S | `signal: string, allowBackend: bool` | Toggle whether the backend can overwrite a signal. |
| `stateChanged` | S -> C | `signal: string, value: f64` | Broadcasts a value change to update UI and Monitor. |

