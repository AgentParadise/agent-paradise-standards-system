---
name: "Notes Service Requirements"
description: "What the notes service must be and do, with measures, check status and user stories"
owner: "Product lead"
status: accepted
version: 1.1.0
last_reviewed: 2026-11-01
serves: north-star.md
---

# Notes Service Requirements

Worked example for APS-V1-0003.RQ01. In a real project this file lives at
`docs/requirements.md` beside `docs/north-star.md`.

## Vision

People capture a thought in under two seconds and find it again in under
ten, on any device, without thinking about where it was saved.

## Roles

- **Writer**: a person who captures and edits notes.
- **Reader**: a person a note is shared with.
- **Agent**: an automated assistant that files, links and summarises notes.
- **Operator**: the person who runs the service.

## Requirements

### P1: Capture

| ID | Requirement | Measure | Check |
|----|-------------|---------|-------|
| P1.1 | A new note opens ready to type from anywhere in the app. | Keystroke to first typed character, p95. | checked: `e2e-capture-latency` |
| P1.2 | No typed text is ever lost, including on crash or lost network. | Crash and offline fault-injection suite loses zero characters. | partial: `test-offline-sync` |

### P2: Find

| ID | Requirement | Measure | Check |
|----|-------------|---------|-------|
| P2.1 | Search returns the note a person is looking for in the first five results. | Top-5 hit rate on the labelled query set is at least 90%. | later |
| P2.2 | Search works offline over every note on the device. | Search succeeds with the network disabled. | unchecked |

### P3: Trust

| ID | Requirement | Measure | Check |
|----|-------------|---------|-------|
| P3.1 | A shared note is readable only by the people it was shared with. | Access-control test matrix passes for every share mode. | checked: `test-share-acl` |

## User Stories

### Writer

- As a Writer, I want a note to open the moment I press the shortcut, so that I keep the thought I just had.
- As a Writer, I want my edits kept when the app crashes, so that I never retype anything.

### Reader

- As a Reader, I want a shared note to open without an account prompt, so that I can read it at once.

### Agent

- As an Agent, I want each note's links and tags in a stable format, so that I can file notes without guessing.

### Operator

- As an Operator, I want every requirement's check named in one place, so that I can see what is proven and what is not.

## Later and Out of Scope

- Later: real-time co-editing of one note by several writers.
- Out of scope: a public publishing platform. Notes are private or shared, never public.

## Change Log

- 1.1.0 (2026-11-01): Added P3.1 (shared-note access). Moved co-editing to Later.
- 1.0.0 (2026-09-15): First accepted version.
