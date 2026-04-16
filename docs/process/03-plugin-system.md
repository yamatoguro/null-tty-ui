# Plugin System

## Design goals
- Hot-swappable behavior.
- Low overhead.
- Strict permissions for safety.
- Stable plugin contract.

## Plugin contract (v1)
- `init(context) -> state`
- `update(state, snapshot) -> view_model`
- `on_event(state, event)`
- `dispose(state)`

## Plugin metadata
- id
- version
- region compatibility
- required permissions
- update interval

## Permission model
- net.read
- fs.read
- shell.exec
- metrics.read

## Priority plugin targets
- system overview
- process table
- network activity
- log stream
- Technitium DNS chart panel
