RELEASE_TYPE: minor

This release adds health check support. A new `HealthCheck` enum and `suppress_health_check()` builder method allow suppressing specific health checks (e.g., `FilterTooMuch`, `TooSlow`, `DataTooLarge`). Health check failures from the server are now reported as panics with descriptive messages.
