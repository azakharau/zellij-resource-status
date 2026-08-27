## Context

The current `zjstatus` configuration can display static mode, tab, and session data from startup, but its command widgets were observed to remain empty when a Zellij session starts in locked mode until the session transitions to normal mode. The desired resource display must work from the default locked startup path and must not require an external shell updater, wrapper daemon, or manual mode switch.

Zellij's plugin API supports timer events and command execution from WASM plugins. Pipes can also deliver rendered text to another plugin and were observed to render in locked mode. The plugin should therefore run inside Zellij, own resource sampling, and publish status output to the bar through a stable integration path.

## Goals / Non-Goals

**Goals:**

- Provide a minimal Rust/Zellij WASM plugin that renders resource status from Zellij startup, including locked mode.
- Render CPU and memory as independent status-bar segments.
- Keep segment backgrounds consistent with the existing status-bar background and color only the measured values by threshold.
- Use low-overhead sampling and bounded update frequency so the plugin's own cost is negligible.
- Integrate with `zjstatus` without requiring a long-running process outside Zellij.
- Include local verification that starts a fresh Zellij session in locked mode, observes the rendered bar, then kills and deletes the test session.

**Non-Goals:**

- Do not add GPU, network, disk, battery, or other resource categories in this change.
- Do not replace the entire status bar.
- Do not depend on shell wrappers, launchd jobs, cron jobs, or user-triggered mode transitions for correctness.
- Do not target non-macOS resource collection in the initial implementation.

## Decisions

### Build a dedicated companion plugin

Implement a small Rust WASM plugin, tentatively named `zellij-resource-status`, rather than relying on `zjstatus` command widgets. The plugin will subscribe to `Timer` events, collect resource metrics, and send formatted status text to `zjstatus` using a named pipe such as `pipe_resources`.

Rationale: `Timer` is part of the Zellij plugin lifecycle and is not tied to normal mode. A companion plugin lets Zellij own the polling lifecycle while preserving the existing `zjstatus` layout and theme.

Alternative considered: Use `zjstatus` `{command_*}` widgets. Rejected because live testing showed empty command output at locked-mode startup.

Alternative considered: Run an external shell updater that sends `zellij pipe`. Rejected as the steady-state design because it adds external moving parts and startup friction.

### Use conservative, macOS-native sampling

CPU sampling will use a low-overhead macOS-native source such as `iostat` or a direct system API if practical in WASM. Memory will derive Activity Monitor-like used memory by excluding reclaimable cached/file-backed pages from total memory. The implementation must avoid `top` polling because it was measured as materially heavier than `iostat`.

Rationale: The status bar should not noticeably increase system load. CPU can tolerate slower update frequency; memory changes more slowly and can be sampled less often.

### Publish formatted status through a named pipe

The companion plugin will emit a single formatted resource string to the existing `zjstatus` pipe target. `zjstatus` will render that string with `pipe_resources_format "{output}"` and `pipe_resources_rendermode "dynamic"`.

Rationale: Pipe delivery was verified to render while the session remains locked. It also keeps status layout ownership in `zjstatus` and resource collection ownership in the companion plugin.

### Keep visual segments separate

The resource output will contain two adjacent, visually independent segments:

- CPU: neutral glyph plus threshold-colored percentage.
- Memory: neutral glyph plus threshold-colored `used/totalG`.

Both segments use the status-bar background, not colored block backgrounds. The session block remains a separate segment after the resource segments.

## Risks / Trade-offs

- [Risk] Zellij WASM plugins may not have direct access to all host system APIs. -> Mitigation: use permitted command execution for small, bounded host commands if direct APIs are not available.
- [Risk] Command execution permissions may prompt or fail if not granted. -> Mitigation: request the minimum required `RunCommand` permission and document the expected permission entry.
- [Risk] Too-frequent sampling can add overhead. -> Mitigation: use timers with conservative intervals and measure `time -l` or equivalent before acceptance.
- [Risk] Pipe destination could be absent if `zjstatus` is not loaded. -> Mitigation: plugin must tolerate missing pipe recipients and retry on the next timer without user-visible errors.
- [Risk] Glyph rendering depends on the terminal font. -> Mitigation: use existing glyph choices from the current status bar and keep fallback labels configurable.
