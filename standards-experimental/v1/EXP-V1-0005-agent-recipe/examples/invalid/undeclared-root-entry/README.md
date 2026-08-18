# Expect: RECIPE_UNDECLARED_ROOT_ENTRY

The recipe root carries `credentials.env`, which is neither a kind this
standard defines nor named in the manifest's `extra_paths`. A recipe MUST
NOT contain credentials (section 2.1); the root is a closed allowlist so
that a secret-bearing directory cannot validate clean.
