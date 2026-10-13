# Knowledge Store: Overview (EXPERIMENTAL)

**EXP-V1-0007** is a placeholder for a standard that does not exist yet: how
to build a personal knowledge store that an agent can read and act on without
ever being handed the owner's private content.

## What is this?

A knowledge store, in this experiment, is three things and one rule:

1. An **append-only log of facts** about knowledge items.
2. A **content-addressed object store** holding the bytes of those items.
3. **Read models** rebuilt from the log, which are the only thing a query ever
   touches.

The rule: **a fact carries a reference, never content.** An append-only log
cannot un-write a payload. If private text ever enters the log, deletion,
access control and safe agent access are all lost at once.

## Why does it matter?

Agents are good at reasoning over the shape of what happened. They should not
need the content to do it, and they should not be able to get the content by
accident. A log that holds only references is one an agent can replay freely;
the bytes live behind a separate port, and resolving a reference is a
deliberate hop that can be authorized, logged, refused, or made impossible.

The same split is what makes the store honest about privacy:

- **Deletion** means removing a blob. The fact stays, the bytes go, and a
  hash chain over the log still verifies.
- **Isolation** means a Compartment (a bounded area of the owner's life) is
  the unit nothing crosses: an item belongs to one, and a reader of one learns
  nothing about another.
- **Tamper evidence** means the log can be hash-chained and its heads anchored
  outside the store, with the content covered through the references it holds.

## Why experimental

One system uses these principles today (DreamShip's Knowledge Core over the
Syntropic137 Event Sourcing Platform). One implementation is a design, not a
standard. The principles are recorded here so a second implementation has a
boundary to build against, and so the open questions are visible while they
are still open. Promotion needs real usage first.

## What this experiment does NOT do

- It ships no validator over a consumer's repository. The `principles`
  command prints the principles; nothing is enforced.
- It does not define an event schema, a wire format, or an API.
- It does not choose a store. The Event Sourcing Platform is the reference
  implementation, not a requirement.
