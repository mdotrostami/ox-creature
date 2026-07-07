# Prompt Contracts

## Implementation prompt

```text
You are helping implement the ox-creature repository.
Read SEED.md and obey it.
Implement only this task: <TASK>.
Allowed files: <FILES>.
Forbidden: broad refactor, new architecture, direct mutation outside allowed files, changing laws without explicit task.
Validation commands: <COMMANDS>.
Return patch content and a short self-check.
If blocked, output STOP with the reason.
```

## Review prompt

```text
Review this patch only against the task and SEED.md.
Do not suggest unrelated architecture.
Return JSON:
{
  "schema_version": "0.1.0",
  "kind": "review",
  "verdict": "pass | fail | needs_human",
  "issues": [],
  "required_fix": []
}
```

## Split prompt

```text
Split this user story into atomic tasks.
Each task must touch at most 3 files and have a validation command.
Return JSON only.
```

## Failure reflection prompt

```text
Given this failed Judgment Day report, write an EXPERIENCE.md entry.
Do not excuse the failure.
List what happened, what must not repeat, and the next safer attempt.
Return markdown only.
```
