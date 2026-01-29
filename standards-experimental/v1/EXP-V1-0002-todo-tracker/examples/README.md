# TODO/FIXME Tracker Examples

This directory contains examples demonstrating the TODO/FIXME tracker standard.

## Sample Repositories

The `sample-repos/` directory contains polyglot code samples with TODO/FIXME comments:

| Directory | Language | Description |
|-----------|----------|-------------|
| `rust/` | Rust | Rust code with various TODO/FIXME patterns |
| `typescript/` | TypeScript | TypeScript/JavaScript examples |
| `python/` | Python | Python code examples |

These samples are used by integration tests to validate the scanner.

## Usage Example

```rust
use std::path::Path;
use todo_tracker::{Scanner, TrackerConfig, ItemSummary, TodoItems, TrackerManifest};

// 1. Create scanner with default config
let config = TrackerConfig::default();
let scanner = Scanner::new(config).expect("Failed to create scanner");

// 2. Scan a repository
let repo_root = Path::new(".");
let result = scanner.scan_repo(repo_root).expect("Failed to scan");

println!("Found {} items in {} files", 
    result.items.len(), 
    result.files_scanned
);

// 3. Generate artifacts
let items = TodoItems::new(result.items.clone());
let summary = ItemSummary::from_items(&result.items, false);

// 4. Serialize to JSON
let items_json = serde_json::to_string_pretty(&items).unwrap();
let summary_json = serde_json::to_string_pretty(&summary).unwrap();

// 5. Write to .todo-tracker/ directory
std::fs::create_dir_all(".todo-tracker").unwrap();
std::fs::write(".todo-tracker/items.json", items_json).unwrap();
std::fs::write(".todo-tracker/summary.json", summary_json).unwrap();
```

## Expected Output

After scanning the sample repositories, you should get artifacts like:

### items.json
```json
{
  "schema_version": "1.0.0",
  "generated_at": "2026-01-21T...",
  "items": [
    {
      "id": "abc123...",
      "tag": "TODO",
      "file": "rust/sample.rs",
      "line": 4,
      "column": 5,
      "text": "TODO(#123): Add integration tests",
      "description": "Add integration tests",
      "issue": {
        "type": "github",
        "number": 123,
        "validated": false
      }
    }
  ]
}
```

### summary.json
```json
{
  "schema_version": "1.0.0",
  "totals": {
    "items": 15,
    "files": 3,
    "tracked": 11,
    "untracked": 4
  },
  "by_tag": {
    "TODO": {"total": 10, "tracked": 7, "untracked": 3},
    "FIXME": {"total": 5, "tracked": 4, "untracked": 1}
  }
}
```

## Testing

The integration tests use these examples:

```bash
cargo test -p todo-tracker
```See `tests/scanner_integration_test.rs` for test cases that validate:
- Polyglot scanning (Rust, TypeScript, Python)
- Issue reference parsing
- Tracked vs untracked items
- Tag distribution
