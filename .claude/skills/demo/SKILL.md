---
name: demo
description: "Guided follow-along for this repo: set up the prerequisites (GitHub CLI + the project scope + a copy you own), install rig, then build the chain from SPEC.md — the foundations first, then kick off the Milestone-1 epic and walk away. Triggers on: 'demo', 'follow along', 'run the demo', 'start the demo', 'set me up', 'walk me through it'."
argument-hint: "(no args — just /demo)"
---

# Follow along: build the chain from its spec

You are guiding a **new person who just cloned this repo** through the rig
workflow end to end, on **their own** machine and GitHub account. The repo
starts as a spec and nothing else; by the end they'll have built the
foundations and kicked off an epic that keeps building on its own.

Be interactive: **check state before each step, explain what you're about to do,
and get a yes before creating anything in their account.** Never touch the
upstream repo (`pgebheim/be-agentic-demo`) — everything lands in the user's own
copy. Stop and wait whenever a step needs them to do something in a terminal
(logins are interactive).

## Step 0 — Confirm the ground

- You should be in a clone of this repo: `SPEC.md` exists at the root. If not,
  ask them to clone it and re-run `/demo`.
- Read `SPEC.md` so you can talk about what you're building (a minimal
  blockchain; Milestone 0 = foundations, Milestone 1 = "a chain that grows").

## Step 1 — GitHub CLI

- `gh --version` — if `gh` isn't installed, point them at
  <https://cli.github.com> and stop until it is.
- `gh auth status` — if not logged in, run `gh auth login` and **stop**; it's
  interactive (browser / device code). Resume when they confirm they're in.

## Step 2 — The `project` scope

The board lives in GitHub **Projects v2**, which needs an extra scope the
default login doesn't grant.

- Check `gh auth status` for `project` in the token scopes.
- If it's missing, have them run **in their own terminal** (not through you —
  it needs a TTY):
  ```
  gh auth refresh -s project --hostname github.com
  ```
  Explain why: "so the workflow can create and move your board." Stop until
  they confirm it completed, then re-check.

## Step 3 — A copy you own

They cloned the upstream, which they can't push to. Give them their own repo:

- Get their login: `gh api user -q .login`.
- Create their repo from this checkout and make it `origin`:
  ```
  gh repo create <you>/be-agentic-demo --private --source . --push
  ```
  (If `origin` already points at the upstream, rename it to `upstream` first so
  `origin` is theirs.) Confirm the new repo URL before continuing.

## Step 4 — Install rig

Onboard rig into their repo — this copies the skills (`rig-plan`, `rig-task`,
`rig-epic`, `rig-tracker`, …) into `.claude/` and writes a starter
`.rig/config.json`. Follow the onboarding skill:
<https://github.com/agent-rig/rig/blob/main/skills/rig-onboard/SKILL.md>
Detect the stack (Rust / cargo). Confirm `cargo` is installed
(<https://rustup.rs>) — the build steps need it.

## Step 5 — A board, wired to the config

- Create the board under their account:
  ```
  gh project create --owner @me --title "be-agentic demo"
  ```
  Capture its **number** from the output.
- Write `.rig/config.json` for their project: `project.repo` = `<you>/be-agentic-demo`,
  `runtime.packageManager` = `cargo`, `test.command` = `cargo test`,
  `tracker.provider` = `github`, and a `tracker.board` block with
  `owner` = their login, `projectNumber` = the number you captured, and
  `statusOptions` matching their board's columns (a fresh board ships **Todo /
  In Progress / Done** — use those). See `rig`'s `docs/tracker-adapter.md`.

## Step 6 — Build the foundations (Milestone 0)

- `/rig-plan SPEC.md --section "Milestone 0"` — decompose it, review the plan,
  then create T1/T2/T3 on the board. (The plan-gate is the point: they approve
  the backlog.)
- Work them: `/rig-task <T1>` then `/rig-task <T2>` (independent), then
  `/rig-task <T3>` (genesis + linking). Let them watch RED → GREEN → review gate
  → PR → merge, and the board move as each lands.
- At the end: `cargo test` is green and there's a two-block chain in a test.

## Step 7 — Kick off the epic, then walk away

Now the payoff — the part they leave running:

- `/rig-plan SPEC.md --section "Milestone 1"` — this milestone is an **epic**
  (interleaved: store → chain-append → node). Approve the plan.
- `/rig-epic run` it. It builds the children in dependency order on a shared
  integration branch, on its own — no one driving.
- Tell them: come back in a while and the board will be further along, and once
  the `node` child lands they can run it and **watch the chain grow**.

## Notes

- Everything happens in **their** repo and account; the upstream is never
  written to.
- Re-runnable: each step checks state first, so `/demo` can be resumed after a
  break or a fixed prerequisite.
- No secrets are stored; the only extra permission is the `project` scope on
  their own `gh` login.
