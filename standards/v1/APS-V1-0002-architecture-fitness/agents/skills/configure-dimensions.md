# configure-dimensions

Set up architectural fitness governance for a project by creating or updating fitness.toml.

## Usage

```
User: "configure fitness" | "set up architectural governance" | "add fitness rules"
```

## Parameters

| Parameter | Required | Default | Description |
|-----------|----------|---------|-------------|
| path | No | `.` | Repository root path |
| project_type | No | auto-detect | `library`, `service`, `monolith`, `microservice`, `frontend` |

## Procedure

1. Check if `fitness.toml` already exists
2. Detect project type from file structure if not specified:
   - Has `Cargo.toml` with `[lib]` → library
   - Has `Dockerfile` or `docker-compose.yml` → service
   - Has `package.json` with frameworks → frontend
   - Has multiple service directories → microservice
3. Generate appropriate configuration:

   **Library projects**:
   - MT01 + MD01 enabled with strict thresholds
   - ST01 for structural checks
   - LG01 for license compliance (critical for libraries)
   - SC01 for vulnerability scanning
   - AC01/PF01/AV01 disabled

   **Service projects**:
   - All default dimensions enabled
   - PF01 recommended if load testing exists
   - AV01 recommended if chaos engineering exists

   **Frontend projects**:
   - MT01 + MD01 + ST01 enabled
   - AC01 enabled (accessibility is critical for frontends)
   - SC01 for dependency vulnerabilities

4. Set dimension weights appropriate for the project type
5. Write `fitness.toml` with commented explanations
6. If `.topology/` exists, run initial validation to show baseline

## Recommended Configurations

### Library (strict quality, strong coupling governance)
```toml
[system_fitness.weights]
MT01 = 0.30    # Libraries must be highly maintainable
MD01 = 0.30    # Coupling discipline is critical for reuse
ST01 = 0.15
SC01 = 0.10
LG01 = 0.15    # License compliance critical for libraries
```

### Service (balanced across all concerns)
```toml
[system_fitness.weights]
MT01 = 0.20
MD01 = 0.20
ST01 = 0.15
SC01 = 0.20    # Security is elevated for services
LG01 = 0.10
PF01 = 0.15    # Performance matters for services
```

### Frontend (user-facing quality focus)
```toml
[system_fitness.weights]
MT01 = 0.20
MD01 = 0.20
ST01 = 0.15
SC01 = 0.10
LG01 = 0.10
AC01 = 0.25    # Accessibility is critical for frontends
```

## Error Handling

| Error | Recovery |
|-------|----------|
| fitness.toml already exists | Ask user if they want to update or replace |
| Cannot detect project type | Ask user to specify |
| No topology artifacts | Warn that validation requires running topology analysis first |
