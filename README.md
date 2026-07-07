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
cargo run -- llm-config-check
cargo run -- status --json
cargo run -- preflight-check
```

## Real Judgment Day

Judgment Day is the GitHub Actions workflow in:

```text
.github/workflows/judgment-day.yml
```

It runs on the first push to `main`, pull requests, mutation branches, and manual dispatch. It produces a GitHub Actions run summary and uploads a Judgment report artifact.

After deterministic checks and LLM model discovery pass, the workflow reaches the `creator-judgment` environment. If that environment has required reviewers configured, GitHub pauses the job and asks the creator to approve/reject from the Actions UI.

The intended creator experience is:

```text
Actions tab
→ Judgment Day run
→ PASS / FAIL summary
→ report artifact
→ Review deployments
→ Approve or reject
```

The creator may approve quickly because the creature is only free inside governed branches. It cannot merge, release, spend secrets, or mutate protected state unless later project phases explicitly implement those actions behind approval gates.


## LLM setup

The non-secret runtime LLM configuration lives in:

```text
config/llm.runtime.json
```

It points to the OpenAI-compatible GapGPT base URL and enables model discovery. The runtime/GitHub workflow may choose the model by discovery policy.

The real API key must **not** be committed. Put it in GitHub Actions secrets with this exact name:

```text
LLM_API_KEY
```

First push flow:

```text
create GitHub repo
→ add LLM_API_KEY secret
→ create creator-judgment environment with yourself as required reviewer
→ push main
→ Judgment Day runs automatically
→ approve/reject from GitHub Actions UI
```

## First proof target

The first real milestone is not full autonomy.

The first milestone is this:

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
