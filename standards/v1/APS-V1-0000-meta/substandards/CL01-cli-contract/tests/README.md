# CLI01 Tests

Tests for the CLI contract are included in `src/lib.rs`.

Run tests with:

```bash
cargo test -p aps-v1-0000-cli01-cli-contract
```

## Test Coverage

- `test_cli_result_success` — Success result has exit code 0
- `test_cli_result_error` — Error result has exit code 1
- `test_cli_result_with_diagnostic` — Adding warning changes status
- `test_diagnostic_with_location` — Diagnostics can have file/line
- `test_command_info` — Command info builder works
- `test_cli_status_exit_codes` — Status maps to correct exit codes
