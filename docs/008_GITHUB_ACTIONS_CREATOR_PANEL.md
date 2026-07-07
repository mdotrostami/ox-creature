# GitHub Actions as Creator Panel

The first creator panel is GitHub Actions itself.

Cockpit will later become the richer visual panel, but the early project must use GitHub's native UI so the creature can meet the creator without needing a large frontend first.

## Required repository setup

Do this before the first push if you want the first push to become the first real Judgment Day.

1. Open repository Settings.
2. Open Secrets and variables → Actions.
3. Add repository secret `LLM_API_KEY`.
4. Open Environments.
5. Create an environment named `creator-judgment`.
6. Add the creator as a required reviewer.
7. Optional: prevent self-review if the same account triggered the run.
8. Push `main`; Judgment Day starts automatically.

## Creator flow

```text
Actions
→ Judgment Day
→ open latest run
→ read summary
→ download artifact if needed
→ Review deployments
→ approve or reject creator-judgment
```

## Button semantics

Approve means:

```text
The creature may continue from this checked branch/run.
```

Reject means:

```text
The creature must stop and record the failure path.
```

## Why this is safe enough for early freedom

The creature can be free in a branch because GitHub Actions and the environment gate are outside the creature's direct control.

It can create, test, and explain mutations.

It cannot truthfully declare final success until the external run records it.
