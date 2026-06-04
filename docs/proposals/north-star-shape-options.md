---
name: "North Star shape options (PV01 restructure)"
description: "Three candidate shapes for the Purpose and Vision substandard restructure with a recommendation, per Addendum 2 item 3"
---

# North Star shape options

The Addendum 2 brief (2026-06-04 22:57) asks PV01 to be restructured into
a single "North Star" section containing Mission, Vision, and Position.
Naming is open. The operator listed three candidate shapes and asked
for 2 or 3 options with a recommendation in the PR body. This file
carries the full reasoning so the choice is auditable; the PR body
includes the short version.

## The three shapes

### Option A: `docs/purpose/` directory of three files

```
docs/
  purpose/
    README.md
    mission.md
    vision.md
    position.md
```

Each file is one of the three sections, indexed in `docs/purpose/README.md`
by the generic parent indexer. The substandard validates that all three
files exist and that each carries the right frontmatter and a single H1.

Strengths:

- Maximum reuse of the generic frontmatter + index mechanism.
- Each section is self contained, embeddable, and searchable on its own.
- A "mission only" project does not have to author the other two; the
  validator can soften per file rather than fail the whole document.

Weaknesses:

- "Purpose" the directory name overloads "Purpose" the concept. The
  operator's framing is North Star, not Purpose.
- Splitting a one-page North Star across three files invites partial
  reads and divergent versioning of three short documents whose value
  is in being read together.
- Adds a level of file system depth for a substandard whose whole point
  is "agents read this on a fresh start".

### Option B: `docs/north-star/` directory of three files

```
docs/
  north-star/
    README.md
    mission.md
    vision.md
    position.md
```

Same shape as Option A but the directory name carries the North Star
framing. Strengths and weaknesses are the same as A with one
improvement (the directory name matches the substandard name) and the
same fragmentation downside.

### Option C (RECOMMENDED): single document at `docs/north-star.md`

```
docs/
  north-star.md      # one document, three required H2 sections
```

The substandard validates one file with required frontmatter and three
required H2 sections (`## Mission`, `## Vision`, `## Position`). The
parent indexer surfaces it by frontmatter `description` the same way it
surfaces every other md file in the docs root.

Strengths:

- One file, one document, one URL to share with operators and agents.
- The North Star is read as a whole. Splitting it weakens it.
- Smallest mechanical delta from the existing PV01 scaffolding: rename
  the file, swap three H2 names, change one default location string.
- Stays at the top of the docs root index where agents bootstrapping
  in a fresh context will see it on first read.
- Honours the "context engineering" framing of EXP-V1-0004: a fresh
  context agent reads one file to know the project's intent.
- The `## Position` section (where the project sits relative to peers
  and competitors) is the smallest of the three and would be a one
  paragraph file under Options A and B. Keeping it inline next to
  Vision is more honest.

Weaknesses:

- Cannot validate Mission only or Vision only without scanning section
  headings (already supported by the existing PV01 implementation, so
  no new cost).
- The full document grows over time. Mitigated by the section discipline
  the validator already enforces.

## Recommendation

Adopt **Option C**.

- Default location: `docs/north-star.md`.
- Required H2 sections, in order: `## Mission`, `## Vision`, `## Position`.
- Substandard slug: `north-star` (the config key is `docs.north-star`).
- Substandard.toml `name`: `"North Star (Mission, Vision, Position)"`.
- Status lifecycle vocabulary unchanged (`proposed` / `active` /
  `deprecated` / `superseded`).
- Diagnostic code renames:
  - `PV01-missing-purpose-section` -> `PV01-missing-mission-section`.
  - `PV01-missing-non-goals-section` -> `PV01-missing-position-section`
    and severity upgraded from warning to error (the brief frames
    Mission, Vision, Position as equal pillars).
  - `PV01-missing-vision-section` stays.

This is the smallest change that lands the operator's restructure and
is easiest to roll back if "Mission, Vision, Position" turns out to be
the wrong vocabulary. Renaming pre merge is cheap: substandard.toml,
spec text, one Rust constant, one default location string. Flag this
choice in the PR body so the operator can override the names before
the spec hits main.

## Items that do not change

- The `.apss/` dot directory stays reserved for generated artifacts
  only (no PV01 config there).
- The substandard's parent (EXP-V1-0004) and the parent's generic
  indexing rules are unaffected. PV01 remains a thin opinion layered on
  the generic mechanism.
- The substandard directory name on disk
  (`PV01-purpose-and-vision/`) stays this round; it is a workspace
  member entry in `Cargo.toml`, referenced from parent specs by relative
  path, and a directory rename is a follow up PR.
