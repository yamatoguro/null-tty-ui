# Technitium Integration

## Objective
Provide a dedicated UI region for DNS query analytics from local Technitium.

## Expected metrics
- queries per minute
- blocked vs allowed
- top domains
- response latency
- cache hit ratio

## Integration strategy
- Poll local Technitium API on interval.
- Normalize data into internal chart model.
- Feed chart plugin with rolling window buffers.

## Extensibility
- Add alert thresholds in config.
- Add custom query tags per zone later.
