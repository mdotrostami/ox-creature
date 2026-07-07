# Cockpit

## Purpose

Cockpit is the Tauri control room where the human sees and steers the creature without reading raw code during normal operation.

## Rule

Cockpit is render-only.

Runtime returns typed presentation blocks. Cockpit renders them and sends human actions back to Runtime.

Cockpit must not decide policy, safety, graph projection, flow semantics, code mutation, or launch eligibility.

## First Cockpit screens

### 1. Identity

Shows:

- seed version,
- permanent laws,
- runtime version,
- current repository,
- current branch,
- current phase.

### 2. Flow Map

Shows:

- bootstrap flow,
- mutation flow,
- judgment flow,
- blocked nodes,
- next allowed action.

### 3. Mission

Shows:

- user story,
- task id,
- intent draft,
- current plan,
- budget,
- blockers.

### 4. Mutation Review

Shows:

- proposal,
- changed files,
- diff summary,
- risks,
- rollback plan,
- approve/reject/request revision.

### 5. Judgment Hall

Shows:

- CI result,
- failed checks,
- artifacts,
- report,
- human verdict.

### 6. Experience

Shows:

- failures,
- repeated patterns,
- reset history,
- rules learned from reality.

## Presentation block contract

The first runtime can emit:

```json
{
  "route": "cockpit.identity",
  "status": "ready",
  "blocks": [
    {
      "type": "law_panel",
      "data": {
        "laws": ["Human Sovereignty", "Reality Before Meaning"]
      }
    }
  ]
}
```

## Human-editable surfaces

The human edits:

- user story,
- scope,
- budget,
- allowed files,
- required gates,
- model tier,
- launch target,
- flow configuration.

The human does not need to edit Rust code for ordinary steering.
