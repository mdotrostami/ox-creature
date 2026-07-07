# Judgment Day

Judgment Day is the creature's external proof ceremony.

It is **not** `cargo run -- preflight-check`.

Local preflight only checks whether the repository is structurally ready to face Judgment Day. Real Judgment Day happens in GitHub Actions.

## Real path

```text
mutation branch / pull request
→ GitHub Actions: Judgment Day
→ deterministic checks
→ LLM model discovery
→ report artifact
→ Actions summary
→ creator-judgment environment
→ creator approval or rejection from GitHub UI
```

## Why GitHub Actions owns it

The creature must not be allowed to grade itself. GitHub provides an external record:

- commit SHA,
- branch,
- workflow run,
- logs,
- uploaded artifacts,
- approval/rejection event,
- protected environment gate.

## LLM discovery

Judgment Day discovers models from the configured provider before reaching creator judgment.

The API key must come from the GitHub secret named `LLM_API_KEY`. The key value must never be written to source, logs, artifacts, issues, or EXPERIENCE.md.

The first discovery target is:

```text
https://api.gapgpt.app/v1/models
```

The creature chooses a model by deterministic policy, preferring Qwen/coder-capable models when available.

## Creator approval model

The workflow includes a job named `creator-judgment`.

That job references the GitHub environment named:

```text
creator-judgment
```

The repository owner should configure that environment with required reviewers. Then GitHub pauses the job until the creator approves or rejects it from the Actions UI.

The creator does not need to inspect raw code for every small mutation. The creature must present:

- summary,
- changed files,
- tests,
- risk flags,
- cost status,
- rollback notes,
- Cockpit block proof.

## Pass

A pass means:

```text
The branch survived Judgment Day and may proceed to the next approved stage.
```

It does not automatically mean production launch unless a later launch workflow exists and is separately approved.

## Fail

A failure means:

```text
The creature stops, preserves evidence, updates or requests update of EXPERIENCE.md, and does not retry blindly.
```

## Forbidden

- local-only self-grading,
- calling preflight Judgment Day,
- hiding failing logs,
- auto-merging without creator gate,
- retrying the same failed strategy without experience update.
