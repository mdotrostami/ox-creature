# Scenario Matrix

## Goal

The creature should not rely on a perfect model. It should survive common failure modes by stopping, validating, or asking.

## Bootstrap scenarios

| Scenario | Expected behavior |
|---|---|
| Fresh repo with SEED.md | `seed-check` passes |
| Missing Human Sovereignty law | fail closed |
| API key in SEED.md | fail closed |
| Missing flow file | flow-check fails |
| Invalid JSON contract | CI fails |

## User story scenarios

| Scenario | Expected behavior |
|---|---|
| Clear small story | create intent candidate |
| Ambiguous story | ask human |
| Story too large | split before coding |
| Story asks for unsafe action | deny or require human review |
| Story exceeds budget | stop before LLM loop |

## LLM scenarios

| Scenario | Expected behavior |
|---|---|
| Valid JSON envelope | parse and validate |
| Extra text around JSON | reject |
| Missing fields | reject |
| Tries to bypass approval | reject and record violation |
| Hallucinates file path | reject during scope check |
| Repeats same failed patch | block until revised plan |

## Git scenarios

| Scenario | Expected behavior |
|---|---|
| On main and wants mutation | create branch first |
| Dirty worktree | stop or require decision |
| Protected branch push | forbidden |
| CI fails | quarantine and write experience |
| CI passes | still require human approval |

## Cockpit scenarios

| Scenario | Expected behavior |
|---|---|
| Runtime status requested | render identity blocks |
| Flow edit requested | create flow-change proposal |
| Mutation review requested | show diff, risk, rollback, gates |
| Judgment result requested | show CI report and next actions |

## Cost scenarios

| Scenario | Expected behavior |
|---|---|
| Cheap deterministic answer available | do not call LLM |
| Near budget limit | stop |
| Expensive model requested for trivial task | downgrade or ask |
| Repeated retries | stop after limit |

## Final meeting scenario

The first meeting succeeds when:

```text
one real user story
→ one branch
→ one small patch
→ CI Judgment Day pass
→ Cockpit-readable report
→ human approve/reject action available
→ outcome recorded
```
