# Cockpit App Placeholder

Cockpit will be a Tauri app, but the first seed does not scaffold Tauri yet.

Reason: the first project must be small enough for a Qwen-like model to advance safely.

Phase 7 will add Tauri after the runtime can already emit valid presentation blocks:

```bash
cargo run -- status --json
```

Cockpit rule:

> Render typed blocks. Send human actions back to runtime. Do not decide policy or mutation safety in frontend.
