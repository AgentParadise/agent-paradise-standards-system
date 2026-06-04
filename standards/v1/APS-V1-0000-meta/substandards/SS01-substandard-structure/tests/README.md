# Tests — APS-V1-0000.SS01

## Test Requirements

Substandard implementations MUST include tests that verify:

### 1. ID Validation

```rust
#[test]
fn test_valid_substandard_ids() {
    assert!(is_valid_substandard_id("APS-V1-0000.SS01"));
    assert!(is_valid_substandard_id("APS-V1-0001.GH01"));
}

#[test]
fn test_invalid_substandard_ids() {
    assert!(!is_valid_substandard_id("APS-V1-0000"));     // Missing suffix
    assert!(!is_valid_substandard_id("APS-V1-0000.ss01")); // Lowercase
}
```

### 2. Parent Reference Extraction

```rust
#[test]
fn test_extract_parent_id() {
    assert_eq!(
        extract_parent_id("APS-V1-0000.SS01"),
        Some("APS-V1-0000".to_string())
    );
}
```

### 3. Metadata Validation

Tests should verify that `substandard.toml` parsing works correctly and validation catches errors.

## Running Tests

```bash
# Run substandard tests
cargo test -p aps-v1-0000-ss01-substandard-structure

# Run via CLI
aps v1 validate substandard APS-V1-0000.SS01
```

