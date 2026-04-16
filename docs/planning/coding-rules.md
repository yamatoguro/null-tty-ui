# Coding Rules

## Documentation rules
- Every public function must have a concise doc comment explaining purpose, inputs, and output.
- Every non-trivial private function must have one short comment describing intent.
- Every background process or loop must include a comment on cadence and stop condition.
- Every method that may fail must document error behavior.

## Naming rules
- Use explicit names for collectors, renderers, adapters, and plugins.
- Avoid short ambiguous names except for loop indexes.

## Structure rules
- Keep modules small and single-responsibility.
- No hidden global mutable state.
- Keep config schema versioned.

## Error handling rules
- Use typed errors.
- Convert external errors at module boundaries.
- Log user-actionable errors with context.

## Performance rules
- Avoid allocations in hot render path.
- Reuse buffers for periodic metrics.
- Cap plugin refresh rates.

## Testing rules
- Unit test parser and config validation.
- Integration test plugin loading.
- Smoke test app startup in headless mode when possible.
