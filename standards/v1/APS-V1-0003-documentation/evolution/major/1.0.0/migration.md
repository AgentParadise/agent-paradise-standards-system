---
name: "Migrating to APS-V1-0003 1.0.0"
description: "How to replace committed CLAUDE.md symlinks with byte-identical copies"
---

# Migrating from APS-V1-0003 0.1.x to 1.0.0

One thing changed: `CLAUDE.md` is a committed, byte-identical **copy** of
the adjacent `AGENTS.md` instead of a symlink to it. See
[`../../../docs/01_spec.md`](../../../docs/01_spec.md) Section 6.4 for the
rule and the reasoning, and `rationale.toml` beside this file for why the
old rule could not be patched.

## 1. Find the symlinks

```bash
git ls-files -s | awk '$1 == "120000"'
```

Every row naming a `CLAUDE.md` is a file this release makes non-conformant.
Check the working tree too, for links that are not yet tracked:

```bash
find . -name CLAUDE.md -type l -not -path './.git/*'
```

## 2. Replace each one

Do NOT `cp AGENTS.md CLAUDE.md` while `CLAUDE.md` is still a symlink. On
Unix, writing through a symlink overwrites its **target**, so that command
destroys the canonical `AGENTS.md` you are trying to preserve. Remove the
link first:

```bash
cd <directory containing the pair>
rm CLAUDE.md            # removes the link itself, not its target
cp AGENTS.md CLAUDE.md  # now a real regular file
git add CLAUDE.md
```

## 3. Verify the committed mode, not the working tree

A working-tree file that looks right can still be recorded as a link:

```bash
git ls-files -s -- CLAUDE.md AGENTS.md
```

Both rows MUST show mode `100644`, and both MUST show the **same blob
hash**. Equal blob hashes are git's own proof of byte-identity; matching
file sizes are not. If `CLAUDE.md` still reports `120000`, force the mode:

```bash
git rm --cached CLAUDE.md && git add CLAUDE.md
git ls-files -s -- CLAUDE.md   # re-check
```

## 4. Keep them equal

`AGENTS.md` is canonical. Edit it, never `CLAUDE.md`. Once the tooling
lands (see below), the pre-commit hook will regenerate `CLAUDE.md` from
`AGENTS.md` and print the discarded diff, and CI will fail on divergence.

## Windows contributors

If you cloned a repository while it still contained a committed symlink and
`core.symlinks=false` (git for Windows' default), your `CLAUDE.md` is a
9-byte text file containing the string `AGENTS.md`. After pulling the
migrated repository, re-checkout the file so the real content lands:

```bash
git checkout -- CLAUDE.md
```

## Tooling status

`apss run documentation claude-md --check` and `--fix` are specified in
Section 6.4.3 but are **not yet implemented**. Until they ship, step 3
above is the manual equivalent of `--check`, and a repository can gate on
it in CI with:

```bash
test "$(git ls-files -s -- CLAUDE.md | awk '{print $1, $2}')" \
   = "$(git ls-files -s -- AGENTS.md | awk '{print $1, $2}')"
```

## What did not change

Nothing in the `docs:` config block, no existing diagnostic code or
severity, and no behaviour of the implemented `validate` or `index`
subcommands. If your repository has no `CLAUDE.md` symlinks, upgrading to
1.0.0 requires no action.
