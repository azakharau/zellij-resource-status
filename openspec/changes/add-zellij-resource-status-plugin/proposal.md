## Why

`zjstatus` command widgets do not render resource data reliably from Zellij startup when the session starts in locked mode. A minimal in-Zellij resource plugin should render CPU and memory status from the first locked-mode frame without relying on external wrapper daemons or mode switches.

## What Changes

- Add a small Zellij WASM plugin that periodically samples system resource data inside the Zellij plugin lifecycle.
- Render independent status segments for CPU and memory, not one combined resource block.
- Keep rendering available in locked mode, normal mode, and all other Zellij modes from startup.
- Use low-overhead sampling and conservative update intervals so the status plugin does not materially contribute to CPU or memory load.
- Integrate with the existing `zjstatus` bar through a pipe or direct compatible output contract.
- Avoid adding extra resource categories beyond CPU and memory in this change.

## Capabilities

### New Capabilities

- `resource-status-rendering`: Rendering and update behavior for CPU and memory resource segments in a Zellij status bar.

### Modified Capabilities

- None.

## Impact

- Adds a Rust/Zellij WASM plugin crate for resource sampling and status output.
- Adds OpenSpec requirements for locked-mode startup rendering, independent CPU and memory segments, low-overhead sampling, and `zjstatus` integration.
- May require Zellij plugin permissions for command execution or host/system state access, depending on the chosen sampling implementation.
