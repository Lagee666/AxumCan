Tech Stack:
* Frontend: HTML, Tailwind CSS, TypeScript (managed via npm)
* Backend communication: WebSockets

Key Features:
* Dashboard Controls: Users can modify CAN signal values via an input field, on-screen up/down buttons, or keyboard up/down arrow keys.
* Dynamic Signals: The dashboard must handle multiple CAN signals fetched from a Rust-based backend.
* Control Arbitrating (Frontend vs. Backend): Introduce a toggle/checkbox that determines whether the backend is allowed to overwrite a signal's value. Changing this toggle should send a boolean flag to the backend to enforce this restriction.
* Real-Time Sync: Use WebSockets to handle bidire

Path:
All web-related code is in dashboard folder.