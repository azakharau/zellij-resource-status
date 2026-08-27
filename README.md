# zellij-resource-status

Minimal Zellij WASM companion plugin that samples macOS CPU and memory from inside the Zellij plugin lifecycle and publishes a `zjstatus` pipe segment.

The plugin is macOS-specific: it reads CPU data from `iostat` and memory data
from `vm_stat` and `sysctl`. Sampling is performed at most once every 10
seconds.

## Build

```sh
scripts/build-wasm.sh
```

The script installs the `wasm32-wasip1` Rust target when needed.

The build writes:

```text
target/zellij-resource-status.wasm
```

## Zellij layout

Load `zjstatus` in the status pane and include the `pipe_resources` widget before `{session}`:

```kdl
pane size=1 borderless=true {
    plugin location="file:/path/to/zjstatus.wasm" {
        format_right "{pipe_resources} #[fg=#cdd6f4]{session}"
        pipe_resources_format "{output}"
        pipe_resources_rendermode "dynamic"
    }
}
```

Load the resource plugin anywhere in the same layout; it hides its own pane after startup:

```kdl
pane borderless=true {
    plugin location="file:/path/to/zellij-resource-status.wasm" {
        pipe_name "pipe_resources"
        interval_secs "10"
        cpu_medium_threshold "70"
        cpu_high_threshold "90"
        memory_medium_threshold "70"
        memory_high_threshold "90"
    }
}
```

The plugin requests the minimum runtime permissions it uses:

- `RunCommands` for `/usr/sbin/iostat`, `/usr/bin/vm_stat`, and `/usr/sbin/sysctl`.
- `MessageAndLaunchOtherPlugins` for publishing `zjstatus::pipe::pipe_resources::...` to `zjstatus` from inside Zellij.

## Verification

```sh
cargo test
cargo build --release --target wasm32-wasip1
scripts/measure_sampler_overhead.sh
ZJSTATUS_WASM=/path/to/zjstatus.wasm scripts/smoke_locked_mode.sh
```

The smoke script creates a disposable locked-mode Zellij session, checks resource output before and after switching to normal mode, then kills and deletes the session.

## License

MIT
