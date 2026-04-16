# Performance and Boot

## Performance budgets (Pi 4 4GB)
- Idle RAM target: <= 350 MB
- Idle CPU target: <= 12%
- Frame target: 30 FPS stable

## Optimization policy
- Prefer static linking where practical.
- Avoid heavy dependencies by default.
- Batch updates to reduce redraw churn.
- Offer quality profiles: low, balanced, high.

## Boot policy
- Launch via systemd user or system service.
- Run in kiosk fullscreen mode.
- Restart on crash with backoff.
- Persist minimal logs for diagnostics.
