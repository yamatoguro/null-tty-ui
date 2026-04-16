# Agent Process Rules

## Mandatory cumulative logging rule
At every interaction that changes project state, append one new entry to `docs/context/cumulative-context.md`.

## Required entry fields
- date
- interaction id (incremental)
- what was done
- how it was done
- why this decision was made
- affected files
- next step

## Enforcement rule
No coding task is complete unless the cumulative context entry for that interaction exists.

## Searchability rule
Use stable section headers and keyword tags in each entry.
