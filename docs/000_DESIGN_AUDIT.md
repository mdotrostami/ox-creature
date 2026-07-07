# Design Audit

## Verdict

The previous pack was a strong myth and blueprint, but not strict enough for a smaller Qwen-like model to advance safely.

This version simplifies the project around one hard rule:

> The model may only propose the next small artifact. The runtime decides whether that artifact is valid.

## Problems fixed

### 1. Too much poetry, not enough contracts

The older pack described the system well, but a weaker model could interpret it many ways.

Fix:

- simple permanent laws,
- JSON flows instead of loose YAML,
- JSON schemas,
- tiny Rust runtime checks,
- explicit scenario matrix,
- strict output envelopes for LLM calls.

### 2. Flow references were ambiguous

Some flow nodes depended on output names instead of node ids.

Fix:

- every `requires` entry now points to an actual node id.
- a flow checker rejects missing references.

### 3. Cockpit was too far away

The previous Cockpit spec was good, but the first repo had no executable proof.

Fix:

- the runtime already emits a Cockpit-style JSON block through `creature status --json`.
- Cockpit starts as a render-only shell later, not as a decision maker.

### 4. CI was too optimistic

The old CI wrote a pass report even when the project barely existed.

Fix:

- CI runs the Rust runtime checks.
- CI validates JSON contracts/flows without installing heavy tools.
- CI writes a report only after checks.

### 5. Reset was conceptual only

Fix:

- `EXPERIENCE_TEMPLATE.md` is included.
- failure scenarios say what must be recorded.

## Simplified architecture

```text
Seed law
→ Runtime checks
→ Flow contracts
→ Git branch
→ Judgment Day CI
→ Cockpit blocks
→ Human decision
→ Experience memory
```

Do not build every domain first. Build the loop first.

## Qwen-safe principle

A Qwen-like model should never receive the whole dream and be asked to build everything.

It should receive one task like:

```text
Implement task 003 only.
Touch max 2 files.
Return patch plus self-check.
Do not invent new architecture.
```

The project must make the correct next step smaller than the model's confusion window.
