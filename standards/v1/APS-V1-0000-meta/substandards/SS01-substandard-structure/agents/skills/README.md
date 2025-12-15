# Agent Skills — APS-V1-0000.SS01

## Substandard Structure Skills

### Skill: Create Substandard

**Trigger**: "Create a substandard for [standard] with profile [code]"

**Steps**:
1. Identify parent standard ID
2. Generate next available profile number
3. Scaffold directory structure
4. Create `substandard.toml` with correct parent reference
5. Add to workspace `Cargo.toml`
6. Run validation

**Example**:
```
User: Create a Python substandard for APS-V1-0001
Agent: Creating APS-V1-0001.PY01...
       ✓ Created substandards/PY01-python/
       ✓ Generated substandard.toml
       ✓ Added to workspace
       ✓ Validation passed
```

### Skill: Validate Substandard

**Trigger**: "Validate substandard [id]"

**Checks**:
- [ ] ID format matches `APS-V1-XXXX.YY##`
- [ ] parent_id matches ID prefix
- [ ] Parent standard exists
- [ ] Located under parent's `substandards/`
- [ ] Required directories present
- [ ] Rust crate compiles

### Skill: Migrate Substandard

**Trigger**: "Update substandard for new parent version"

**Steps**:
1. Check parent version changes
2. Identify breaking changes
3. Update `parent_major` field
4. Apply necessary code changes
5. Bump substandard version
6. Update documentation

