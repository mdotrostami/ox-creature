# Self-Build Engine

Self-build is the creature's normal life loop.

It is not Judgment Day.

## Loop

```text
GitHub Action starts
→ Rust runtime validates the seed, flows, contracts, state, and LLM config
→ runtime emits a bounded self-build context
→ LLM chooses one tiny missing product cell
→ LLM returns JSON only, following contracts/llm-next-action.schema.json
→ Rust runtime validates the proposed mutation
→ Rust runtime writes the allowed files
→ GitHub Action runs fmt/check/test/preflight
→ if valid, the creature commits and pushes itself
→ after config/self-build.json.loop_delay_seconds, the workflow dispatches the next cycle
```

`loop_delay_seconds` defaults to `0`. Change it later to slow the creature down.

## Stop condition

The loop stops only when the creature creates `state/judgment-ready.json` and the runtime verifies required product cells:

- `contracts/user-story.schema.json`
- `contracts/materialized-patch.schema.json`
- `apps/cockpit/README.md`
- `state/judgment-ready.json`

Then the self-build loop requests Judgment Day and stops.

## Authority

LLM output is advisory. The Rust runtime decides what is accepted.

The creature may mutate normal product files, but it may not mutate `.github/` workflows during self-build. Workflow changes are human patches.
