# Technical Architecture

## Core stack
- Language: Rust
- UI: Slint (native rendering path)
- Runtime model: single binary + external config + plugin scripts

## Runtime layers
1. Core app shell: lifecycle, event loop, config load, plugin scheduler.
2. Data collectors: cpu, memory, disk, network, temperature, logs.
3. Terminal bridge: PTY-backed shell session.
4. Layout engine: region map from config.
5. Plugin engine: script plugins + future native plugin ABI.

## Data flow
- Collectors publish snapshots on an internal event bus.
- Plugins subscribe to event topics.
- Layout engine binds plugin outputs to regions.
- Renderer updates visible widgets on a fixed frame budget.
