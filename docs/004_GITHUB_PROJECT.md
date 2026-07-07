# GitHub Project Model

ox-creature treats GitHub as the outer accountability system.

## Git primitives

- `main` is protected.
- mutations happen on `mutation/*` branches.
- every meaningful mutation should become a pull request before merge.
- every run must preserve CI logs and report artifacts.

## GitHub Actions

The first official workflow is:

```text
.github/workflows/judgment-day.yml
```

It runs on the first push to `main`, on `mutation/**` branches, on pull requests to `main`, and through manual dispatch.

It uses official actions:

- `actions/checkout@v6` to check out the repository in the GitHub workspace.
- `actions/upload-artifact@v7` to upload Judgment reports.

## Creator panel

The first creator panel is not Tauri.

The first creator panel is GitHub Actions:

```text
Actions → Judgment Day → Review deployments → approve/reject creator-judgment
```

## Required environment

Create a GitHub environment named:

```text
creator-judgment
```

Configure required reviewers so the final `creator-judgment` job waits for the creator.

## Later GitHub project automation

Later phases may add:

- issue creation on failure,
- PR comments,
- labels,
- project board updates,
- release workflow after creator approval.

These are intentionally not in the first seed because write permissions should be added one by one.
