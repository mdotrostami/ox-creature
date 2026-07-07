# Implementation Plan

## Rule

Do not build the universe first.

Build the smallest loop that can survive Judgment Day.

## Phase 0 — Lock the seed

Tasks:

- validate `SEED.md`,
- detect obvious secrets,
- validate flows,
- validate contracts,
- print Cockpit status blocks.

Demo:

```bash
cargo run -- seed-check
cargo run -- flow-check
cargo run -- status --json
```

## Phase 1 — Git spine

Tasks:

- detect current branch,
- detect dirty worktree,
- create safe mutation branch,
- write branch metadata artifact.

Demo:

```bash
cargo run -- git-status --json
```

## Phase 2 — User story intake

Tasks:

- accept a story,
- create intent candidate JSON,
- reject ambiguous/large story,
- create task artifact.

Demo:

```bash
cargo run -- story "Add a status command" --budget 0.25
```

## Phase 3 — Plan before patch

Tasks:

- create small plan object,
- require explicit scope,
- show Cockpit review block.

## Phase 4 — Patch artifact

Tasks:

- build patch artifact,
- enforce allowed files,
- require rollback note,
- apply only on branch.

## Phase 5 — Judgment Day

Tasks:

- run CI,
- upload report,
- show report in Cockpit,
- require human approval for merge.

## Phase 6 — Experience memory

Tasks:

- write real outcome,
- update `EXPERIENCE.md`,
- detect repeated failures,
- support Genesis Reset.

## Phase 7 — Tauri Cockpit

Tasks:

- create Tauri shell,
- render presentation blocks,
- send human actions to runtime,
- never make policy decisions in frontend.
