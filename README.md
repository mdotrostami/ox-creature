# ox-creature

`ox-creature` is a small, GitHub-native seed for a self-understanding software creature.

It is **not** part of OntologyX/OX. It is a separate GitHub project.

The creature starts from three things:

1. `SEED.md` — permanent law.
2. A tiny Rust runtime — deterministic local checks and Cockpit presentation blocks.
3. GitHub — branch history, issues, PRs, Actions, environment approvals, and Judgment Day.

Core rule:

> The model may think. The runtime, GitHub, CI, and the human creator decide what becomes real.

## First local commands

Local commands are not Judgment Day. They are only genesis/preflight checks.

```bash
cargo run -- seed-check
cargo run -- flow-check
cargo run -- contract-check
cargo run -- status --json
cargo run -- preflight-check
```

## Self Loop vs Judgment Day

`ox-creature` has two different GitHub-native rituals:

```text
.github/workflows/self-loop.yml       # ordinary life / repeated growth
.github/workflows/judgment-day.yml    # rare final trial / creator meeting
```

**Self Loop** is the normal repeating workflow. It may run on push or manual dispatch. It checks the seed, contracts, flows, runtime status, and project hygiene. It may later open issues, propose the next small mutation, or prepare a branch. Passing Self Loop means the creature is still healthy enough to continue learning.

**Judgment Day** is not the normal loop. It is called only when the creature itself claims it is ready to meet the creator: it must prove that a user story can become safe, working, visible, explainable, affordable code, with Cockpit-readable proof. Judgment Day is manually dispatched by the creature or the creator, and waits at the `creator-judgment` environment gate.

The intended creator experience is:

```text
Actions tab
→ Self Loop keeps repeating during growth
→ creature eventually announces Judgment Day readiness
→ Judgment Day run
→ PASS / FAIL summary and report artifact
→ Review deployments
→ Approve or reject
```

Until Judgment Day is explicitly called, green runs are only Self Loop health signals, not final proof.

## First proof target

The first real milestone is not Judgment Day.

The first milestone is the Self Loop staying alive while it prepares this path:

```text
User story
→ small validated intent
→ small plan
→ small patch on a branch
→ CI Judgment Day
→ Cockpit-readable report
→ creator approval or rejection
→ memory of the result
```

## Non-negotiables

- The human remains sovereign forever.
- The creature is free to mutate only inside governed branches.
- The creature never pushes directly to protected `main`.
- LLM output is advisory until parsed, validated, and approved.
- Every mutation happens on a branch.
- Every pass/fail creates a readable artifact.
- If in doubt, stop.
- If it fails repeatedly, reset from `SEED.md` + `EXPERIENCE.md`, not from vibes.

## Repository shape

```text
SEED.md
Cargo.toml
src/main.rs
contracts/
flows/
docs/
apps/cockpit/
.github/workflows/judgment-day.yml
```

The first runtime intentionally has almost no dependencies. This keeps the early project cheap, inspectable, and friendly to smaller coding models.
