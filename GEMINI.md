# CAN Simulator Project Knowledge Base

## 1. System Architecture
*   **Backend:** Rust with `Axum` (Web server), `Tokio` (Async runtime), and `can-comm` (CAN protocol stack).
*   **Frontend:** Single Page Application (SPA) using `TypeScript`, `Vite`, and `Tailwind CSS 4.0`.
*   **Communication:** Full-duplex WebSockets for real-time signal synchronization and control arbitration.

## 2. WebSocket Protocol (JSON)
All messages use a `type` discriminator and `camelCase` naming.

| Message Type | Direction | Payload | Description |
| :--- | :--- | :--- | :--- |
| `init` | S -> C | `signals: Signals` | Sends the full initial signal map to the client. |
| `clientUpdate` | C -> S | `signal: string, value: f64` | User changed a value manually on the dashboard. |
| `setArbitration`| C -> S | `signal: string, allowBackend: bool` | Toggle whether the backend can overwrite a signal. |
| `stateChanged` | S -> C | `signal: string, value: f64` | Broadcasts a value change to update UI and Monitor. |

## 3. Backend Implementation (`Registry` Pattern)
The `Registry` struct is the central coordinator:
*   **`update_dashboard(label, value)`**: Used for signals that exist in the control grid. It respects arbitration toggles, updates CAN actors, and syncs the UI of all connected clients.
*   **`send_to_monitor(name, value)`**: Used for passive logging. It accepts any `String` name and only updates the frontend's "Monitor" panel without affecting CAN actors or control inputs.
*   **Arbitration:** Managed via `Arc<Mutex<HashMap<SignalLabel, bool>>>` to prevent the backend from fighting with a user's manual inputs.

## 4. Frontend Conventions
*   **Multi-View Navigation:** Uses a `switchPage` function to toggle visibility between `view-controls` (Grid of inputs) and `view-monitor` (Scrolling log).
*   **DOM Injection:** Signals are rendered dynamically on `init`. Each signal has a unique ID used for targeted UI updates when a `stateChanged` message arrives.
*   **Global Control:** A header toggle provides a bulk action to enable/disable backend control for every signal simultaneously.

## 5. Critical Technical Lessons (Pitfalls to Avoid)
*   **Axum 0.8+ Routing:** Do NOT use `.nest_service("/", ...)` at the root. Use `.fallback_service(ServeDir::new("..."))` to serve static assets alongside WebSocket routes.
*   **Serde Renaming:** `#[serde(rename_all = "camelCase")]` on a Rust enum only renames the variants. Fields inside struct variants (e.g., `allow_backend`) must be explicitly renamed using `#[serde(rename = "allowBackend")]` to match TypeScript.
*   **Tailwind 4.0:** Requires `@import "tailwindcss";` in the CSS file and the `@tailwindcss/postcss` plugin instead of the legacy `tailwindcss` PostCSS plugin.
*   **WebSocket URLs:** Use `window.location.host` to dynamically determine the WebSocket connection string, ensuring it works across different network environments (localhost, IP, or WSL).
