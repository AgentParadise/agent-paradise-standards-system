# CLI01 Examples

## Implementing StandardCli

See the Code Topology standard (EXP-V1-0001) for a complete implementation.

### Basic Implementation

```rust
use aps_v1_0000_cli01_cli_contract::{
    StandardCli, CliResult, CliCommandInfo, CliDiagnostic
};

pub struct MyStandardCli;

impl StandardCli for MyStandardCli {
    fn slug(&self) -> &str { "my-standard" }
    fn id(&self) -> &str { "APS-V1-0001" }
    fn aps_version(&self) -> &str { "v1" }
    fn name(&self) -> &str { "My Standard" }
    fn description(&self) -> &str { "Does something useful" }
    fn version(&self) -> &str { "1.0.0" }
    
    fn commands(&self) -> Vec<CliCommandInfo> {
        vec![
            CliCommandInfo::required("validate", "Validate artifacts"),
            CliCommandInfo::optional("analyze", "Generate artifacts"),
        ]
    }
    
    fn execute(&self, command: &str, args: &[String]) -> CliResult {
        match command {
            "validate" => self.validate(args),
            "analyze" => self.analyze(args),
            _ => CliResult::error(command, format!("Unknown command: {command}")),
        }
    }
}

impl MyStandardCli {
    fn validate(&self, _args: &[String]) -> CliResult {
        CliResult::success("validate")
    }
    
    fn analyze(&self, _args: &[String]) -> CliResult {
        CliResult::success("analyze")
            .with_diagnostic(CliDiagnostic::info("GENERATED", "Created 5 artifacts"))
    }
}
```

### Usage

```bash
# Run the standard
aps run my-standard validate .
aps run my-standard analyze . --output .artifacts/
```
