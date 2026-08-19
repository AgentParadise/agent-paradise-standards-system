# Security Judge Prompt

Given the agent's review output and the eval case's `expected.md`, judge
whether the review identifies the same security issue `expected.md`
describes, using the same severity or greater. Do not reward a review that
raises unrelated concerns instead of the one under test.
