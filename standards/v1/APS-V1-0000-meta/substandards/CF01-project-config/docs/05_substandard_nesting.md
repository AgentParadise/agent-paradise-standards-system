# APS-V1-0000.CF01: Substandard Nesting Convention (Normative)

**Version**: 1.0.0
**Status**: Active
**Parent**: APS-V1-0000.CF01 (Project Configuration)

Sibling normative spec to `01_spec.md`, `02_slug_registry.md`,
`03_contribution_schema.md`, and `04_validation_delegation.md`.
Equal precedence under APS-V1-0000 §1.1.

## Terminology

RFC 2119 keywords apply.

---

## 1. Decision: Nested-Key Convention

Substandards MUST be configured as nested keys under the parent
standard's slug. Substandards MUST NOT receive a top-level slug in
APSS.yaml.

```yaml
docs:
  enforce_adr: true
  adr:
    disable: false
    adr_dir: docs/adrs
  purpose-and-vision:
    disable: true
  retrospectives:
    cadence: monthly
```

This is the form already implied by the slug registry (see
`02_slug_registry.md` §2.2.1 and §3.3): every substandard appears
inside its parent's `substandards` array, keyed by the substandard's
slug.

---

## 2. Why Nested, Not Top-Level

The operator's starting recommendation in the design brief was to
nest substandards under the parent slug. This spec adopts that
recommendation and records the supporting argument so future readers
can evaluate it.

### 2.1 Three Alternatives Considered

1. **Top-level qualified key.** Examples: `docs.adr:`, `docs.pvs:`.
   Rejected because TOML keys containing dots are syntactically legal
   but read poorly, conflict with editor schema completion (which
   expects a single token), and force the meta-validator to do string
   surgery on every top-level key to discover parents.
2. **Top-level fresh slug per substandard.** Examples: `adr:`,
   `pvs:`. Rejected because:
   - it pollutes the slug namespace (substandards multiply faster than
     standards),
   - it loses the grouping signal that "ADR enforcement is part of the
     docs standard",
   - it requires substandards to declare uniqueness across the entire
     project, not just within their parent (raising the bar for
     experiment authors),
   - it makes "disable all of docs" impossible without disabling each
     substandard individually.
3. **Nested keys under the parent.** Adopted.

### 2.2 Why Nested Wins

- **Grouping is correct.** Substandards are scoped to their parent in
  the meta-standard (APS-V1-0000 §4.2). Nesting in APSS.yaml matches
  that hierarchy without lossy flattening.
- **Disable inheritance is natural.** `disable: true` at the parent
  slug means the parent and all its substandards are off. The
  meta-validator does not need to walk a separate cascade.
- **Slug namespace stays small.** Only standards and experiments
  compete for top-level slugs; substandards compete only within their
  parent. That keeps the namespace governable as the ecosystem grows.
- **Editor schema is straightforward.** The aggregate
  `generated/v1/apss.schema.json` (see `03_contribution_schema.md`
  §6.2) is a flat union of slugs, each with a nested object of
  substandard slugs. Completion works out of the box.
- **It matches existing precedent.** EXP-V1-0004's `.apss/config.toml`
  already nested substandard toggles under the parent section; this
  preserves operator muscle memory across the migration.

The single small cost is that a substandard cannot be configured
without naming its parent. That cost is paid in APSS.yaml only, where
it actually helps the reader.

---

## 3. The Nested Shape

### 3.1 Structural Form

Inside a parent standard's section, a key MAY name either:

- a standard-owned property declared in the parent's contribution
  schema (e.g. `enforce_adr`, `adr_dir`), OR
- a substandard slug declared in the parent's slug registry entry
  (e.g. `adr`, `retrospectives`).

The meta-validator distinguishes them by consulting the contribution
schema. Standard-owned properties have leaf values or arrays;
substandard slugs always map to objects whose schema is contributed
by the substandard. The two namespaces share the parent's key space
and MUST NOT collide; see §3.4.

### 3.2 Universal Keys Inside Substandards

Each substandard section MAY contain the same universal keys as a
standard section (see `03_contribution_schema.md` §3.1):

| Key | Type | Default | Purpose |
|-----|------|---------|---------|
| `disable` | bool | inherits from parent | Disables this substandard. |
| `version` | string | inherits from parent | Reserved for future per-substandard version pinning. |

Inheritance rule: a substandard's effective `disable` is
`parent.disable OR substandard.disable`. A parent that is disabled
implicitly disables all its substandards; a substandard MAY be
disabled while its parent is active.

The `version` key at substandard scope is reserved but not yet
specified; CF01 v1 ignores it. Implementations MUST NOT reject it
as unknown.

### 3.3 Default-On Holds at Substandard Scope

An active substandard requires no nested section. A substandard
section is needed only to override defaults or to toggle `disable`.
This carries the default-on philosophy from `01_spec.md` §5 down to
substandard scope without modification.

### 3.4 Collisions Between Standard Keys and Substandard Slugs

If a standard's contribution schema declares a property with the
same name as one of its substandard slugs, the meta-validator MUST
emit `CF_SUBSTANDARD_KEY_SHADOW`. This is a build-time error caught
during `<bootstrap> v1 validate repo`, not a runtime error against
APSS.yaml. Standards are responsible for avoiding the clash; the
slug registry produces the canonical list of substandard slugs
each standard owns.

| Code | Severity | Rule |
|------|----------|------|
| `CF_SUBSTANDARD_KEY_SHADOW` | Error | Standard contribution schema declares a property with the same name as a substandard slug. |

This rule exists so that consumers can read APSS.yaml without ever
having to disambiguate "is this a leaf property or a substandard?".

---

## 4. Disable Semantics

The combined disable matrix:

| Parent | Substandard | Effective state |
|--------|-------------|-----------------|
| absent | absent | parent and substandard active with defaults |
| absent | `disable: false` | identical to above (documentation only) |
| absent | `disable: true` | parent active, this substandard inactive |
| `disable: false` | absent | identical to all-absent |
| `disable: false` | `disable: true` | parent active, this substandard inactive |
| `disable: true` | absent | parent and all substandards inactive |
| `disable: true` | `disable: false` | parent and all substandards inactive (parent wins) |

The "parent wins" rule for the last row is the deliberate choice.
A substandard MUST NOT be able to re-enable itself once its parent
is off. The alternative (substandard wins) lets a substandard
silently activate even though its parent is supposed to be off,
which is the exact failure mode of cascade-without-discipline.

If a project genuinely needs the substandard only, it MUST enable
the parent and disable the parent's other substandards explicitly.

---

## 5. Worked Examples

### 5.1 All Defaults

```yaml
schema: apss.project/v1

project:
  name: my-service
  apss_version: v1
```

The docs standard is active. All docs substandards (ADR, PVS,
retrospectives) are active with their defaults. No section needed.

### 5.2 Override One Substandard

```yaml
docs:
  adr:
    adr_dir: docs/decisions
```

Docs and all substandards are still active; only `adr_dir` is
overridden for the ADR substandard.

### 5.3 Disable One Substandard, Keep Parent

```yaml
docs:
  purpose-and-vision:
    disable: true
```

Docs is active; ADR and retrospectives are active; PVS is off.

### 5.4 Disable the Whole Standard

```yaml
docs:
  disable: true
```

Docs and all substandards are off. The runtime skips them entirely.
The meta-validator still runs structural validation against any
remaining keys under `docs:`, so a typo like `purpse-and-vision`
would still be caught.

### 5.5 Disable Substandard Despite Disabled Parent

```yaml
docs:
  disable: true
  adr:
    disable: false   # ignored; parent wins per §4
```

The meta-validator MUST emit `CF_SUBSTANDARD_REENABLE_IGNORED`
(warning) so the consumer knows their attempt was no-op:

| Code | Severity | Rule |
|------|----------|------|
| `CF_SUBSTANDARD_REENABLE_IGNORED` | Warning | Substandard sets `disable: false` while its parent has `disable: true`; the substandard remains disabled. |

---

## 6. Cross-References

- Slug rules for substandards: see `02_slug_registry.md` §3.3.
- Schema contribution for substandards: see
  `03_contribution_schema.md` §4.
- How the meta-validator dispatches substandard validation: see
  `04_validation_delegation.md` §2 (steps 6 to 8) and the
  `DelegatedSection` contract.
- How `disable: true` interacts with the unified installer: see
  `06_unified_install_seam.md` §4 (removed standards uninstall
  their hooks cleanly; removed substandards uninstall theirs).
