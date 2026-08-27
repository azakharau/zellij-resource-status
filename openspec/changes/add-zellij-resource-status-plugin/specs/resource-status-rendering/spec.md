## ADDED Requirements

### Requirement: Render resources from locked-mode startup

The plugin SHALL render resource status in a fresh Zellij session that starts in locked mode without requiring a switch to normal mode or any user input.

#### Scenario: Fresh locked session shows resources

- **WHEN** a new Zellij session starts with `default_mode "locked"` and the resource plugin loaded in the status layout
- **THEN** CPU and memory segments are visible in the status bar without switching modes

#### Scenario: Resource status remains visible across modes

- **WHEN** the user switches between locked, normal, pane, tab, resize, scroll, and session modes
- **THEN** the CPU and memory segments remain visible unless the status-bar pane itself is removed

### Requirement: Render CPU and memory as separate segments

The plugin SHALL provide CPU and memory as separate status-bar segments rather than one combined resource block.

#### Scenario: Separate resource segments

- **WHEN** the status bar renders resource information
- **THEN** the CPU segment appears before the memory segment and both appear before the session segment

#### Scenario: Segment content format

- **WHEN** resource data is available
- **THEN** the CPU segment shows a neutral CPU glyph and a percentage value
- **THEN** the memory segment shows a neutral memory glyph and a `used/totalG` value

### Requirement: Use threshold colors only for values

The plugin SHALL keep resource segment backgrounds equal to the status-bar background and SHALL color only the measured values by threshold.

#### Scenario: Low usage color

- **WHEN** CPU or memory usage is below its low threshold
- **THEN** the measured value is rendered in the configured green color

#### Scenario: Medium usage color

- **WHEN** CPU or memory usage is at or above its medium threshold and below its high threshold
- **THEN** the measured value is rendered in the configured yellow color

#### Scenario: High usage color

- **WHEN** CPU or memory usage is at or above its high threshold
- **THEN** the measured value is rendered in the configured red color

#### Scenario: Neutral glyphs

- **WHEN** resource segments render
- **THEN** resource glyphs are rendered in the configured neutral foreground color
- **THEN** resource segment backgrounds are not rendered as colored warning blocks

### Requirement: Minimize plugin overhead

The plugin SHALL collect and publish resource data with bounded overhead that is negligible compared with normal terminal usage.

#### Scenario: Conservative update cadence

- **WHEN** the plugin is running continuously
- **THEN** CPU sampling occurs no more frequently than once every 10 seconds by default
- **THEN** memory sampling occurs no more frequently than once every 10 seconds by default

#### Scenario: Avoid heavy polling commands

- **WHEN** the plugin collects CPU or memory data on macOS
- **THEN** it does not use `top` as the default polling mechanism

#### Scenario: Overhead is measured before acceptance

- **WHEN** implementation is complete
- **THEN** verification includes a measurement of the sampler command or API overhead
- **THEN** the result is documented in the implementation evidence

### Requirement: Integrate with zjstatus through an in-Zellij path

The plugin SHALL integrate with the existing `zjstatus` bar without relying on an external long-running process outside Zellij.

#### Scenario: Pipe integration

- **WHEN** `zjstatus` is configured with a resource pipe placeholder
- **THEN** the resource plugin publishes formatted resource output to that pipe from inside the Zellij plugin lifecycle

#### Scenario: Missing bar recipient

- **WHEN** the resource pipe recipient is not available
- **THEN** the plugin does not crash
- **THEN** it retries publication on the next scheduled update

### Requirement: Verify with real Zellij sessions

The implementation SHALL be verified against real Zellij sessions, not only by static config inspection.

#### Scenario: Automated smoke session

- **WHEN** verification runs
- **THEN** it creates a fresh test Zellij session
- **THEN** it confirms CPU and memory render in locked mode
- **THEN** it kills and deletes the test session before completing
