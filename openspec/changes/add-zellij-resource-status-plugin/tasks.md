## 1. Repository Scaffold

- [x] 1.1 Create a minimal Rust plugin crate targeting Zellij WASM.
- [x] 1.2 Add build scripts or documented commands for producing the `.wasm` artifact.
- [x] 1.3 Add a compact README with install, config, and verification commands.

## 2. Plugin Lifecycle

- [x] 2.1 Implement plugin `load` to request the minimum required permissions.
- [x] 2.2 Subscribe to Zellij timer events and schedule the first update during startup.
- [x] 2.3 Ensure the first resource publication happens without requiring normal mode.
- [x] 2.4 Reschedule periodic updates after every timer event.

## 3. Resource Sampling

- [x] 3.1 Implement low-overhead CPU sampling without using `top` as the default mechanism.
- [x] 3.2 Implement macOS memory sampling using Activity Monitor-like used memory semantics.
- [x] 3.3 Format memory as `used/totalG`.
- [x] 3.4 Apply configurable threshold colors to values only.
- [x] 3.5 Keep glyphs neutral and segment backgrounds equal to the status-bar background.

## 4. Status-Bar Integration

- [x] 4.1 Publish resource output to a named `zjstatus` pipe such as `pipe_resources`.
- [x] 4.2 Document the matching `zjstatus` configuration snippet.
- [x] 4.3 Render CPU before memory and both before the session segment.
- [x] 4.4 Tolerate missing or delayed `zjstatus` recipients and retry on the next update.

## 5. Verification

- [x] 5.1 Add a smoke test or script that starts a fresh Zellij session in locked mode.
- [x] 5.2 Verify CPU and memory segments render in locked mode without switching to normal mode.
- [x] 5.3 Verify resource segments remain visible after switching modes.
- [x] 5.4 Measure sampler overhead and record the result in implementation evidence.
- [x] 5.5 Kill and delete every test Zellij session created by verification.
