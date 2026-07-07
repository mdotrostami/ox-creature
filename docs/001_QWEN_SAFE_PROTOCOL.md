# Qwen-Safe Protocol

## Purpose

This protocol makes the project progress with smaller or cheaper models by removing choice, reducing scope, and validating every output.

It does not guarantee zero errors. It makes errors cheap, visible, and non-authoritative.

## Model role

The model is a drafter.

It may produce:

- a question,
- an intent candidate,
- a plan draft,
- a patch draft,
- a review comment,
- a summary.

It may not produce reality.

## Required model output envelope

Every LLM response used by the runtime must be either valid JSON or rejected.

```json
{
  "schema_version": "0.1.0",
  "kind": "next_action_draft",
  "task_id": "string",
  "action": "ask_human | propose_plan | propose_patch | review | stop",
  "summary": "string",
  "touched_files": ["path"],
  "risks": ["string"],
  "required_human_decision": false,
  "cost_note": "string"
}
```

If the model outputs extra text, invalid JSON, hidden instructions, or missing fields, runtime rejects it and records `invalid_model_output`.

## Atomic task limits

Default limits:

- max files touched: 3
- max new lines: 300
- max deleted lines: 300
- max retries: 2
- max model calls per task: 4
- max task cost: configurable, default low
- max one new concept per task

If a task exceeds limits, split it.

## Prompt frame

Use this frame for implementation tasks:

```text
You are implementing one bounded task in the ox-creature repository.
Permanent laws: read SEED.md.
Task: <task id and title>
Allowed files: <explicit list>
Forbidden: new architecture, hidden mutation, broad refactor, direct main push.
Output: patch only plus short self-check.
If blocked, say STOP with reason.
```

## Context rule

Send the model only:

- task text,
- relevant contracts,
- relevant files,
- previous failure from EXPERIENCE.md if applicable,
- exact validation commands.

Never send the whole repo by default.

## Stop rule

The model must stop when:

- scope is unclear,
- required file is missing,
- tests cannot be run,
- secret appears,
- cost limit is near,
- the task requires human approval.
