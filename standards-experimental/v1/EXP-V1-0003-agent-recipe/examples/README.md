# Agent Recipe Standard Examples

This directory contains examples demonstrating the experimental standard, and is
loaded directly by the crate's tests via `include_str!`.

## Available Examples

| Example | Description |
|---------|-------------|
| `valid/full.yaml` | All fields populated: skills and a system instruction override |
| `valid/minimal.yaml` | Only the required fields: `name`, `agent`, `model.name`, `model.effort` |
| `invalid/missing-required-fields.yaml` | Missing `agent` and `model`, empty `name` |
| `invalid/unknown-agent.yaml` | `agent: gemini`, not a recognized v1 harness |
| `invalid/unknown-field.yaml` | A field (`temperature`) not part of the schema |
| `invalid/invalid-effort-and-mode.yaml` | Bad `model.effort` and `system_instructions.mode` values |

Each invalid example documents, in a leading comment, which error code(s) it is
expected to trigger.

## Purpose

Examples in experiments serve to:
1. Demonstrate proposed patterns
2. Gather feedback from users
3. Validate the approach before promotion
4. Exercise the validator's error-code coverage in automated tests

