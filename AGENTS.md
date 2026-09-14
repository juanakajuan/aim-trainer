# Project instructions

## Questions and communication

- Use the available question tool for user decisions.
- Be concise. Use ASD-STE100 Simplified Technical English.

## Type safety and tests

- Keep strict types. Define explicit types for public APIs and data structures.
- Validate external data at boundaries.
- Do not use unchecked casts, type suppression, or weaker compiler checks.
- Keep `unsafe_code = "forbid"`. Treat Clippy warnings as errors during checks.
- Add tests only for crucial behavior, unless the user requests more tests.

## AI workflow

Use `$ai-workflow` only when the user invokes it. The personal skill is at
`/home/juanix/.dotfiles/.agents/skills/ai-workflow/SKILL.md`.

Settings agreed on 2026-09-14:

- Repository: https://github.com/juanakajuan/aim-trainer (public).
- Remote: `origin`, `git@github.com:juanakajuan/aim-trainer.git`.
- PR base: `main`.
- Main checkout: `/home/juanix/Projects/aim-trainer`.
- Task branch: `task/<issue>-<slug>`.
- Task worktree: `/home/juanix/Projects/aim-trainer-worktrees/<issue>-<slug>`.
- Use Git to create branches and worktrees from the current `origin/main`.
- Keep the main checkout available. Preserve existing changes.
- Record scope and acceptance criteria in a GitHub issue. Get explicit approval
  for that issue and scope before implementation.
- Use small conventional commits. Open a draft PR after the first useful commit.
- Keep progress, restart data, check results, review results, and repair counters
  in the PR description. The issue remains the scope record.
- Allow at most three repair attempts per failed check and three review repair
  rounds after the initial failure or review. Stop at the limit.
- Before readiness, fetch the base. If it advanced, merge it and repeat affected
  checks and fresh review. Record the verified commit and base in the PR.
- The user reviews and squash-merges in GitHub. Do not merge or enable auto-merge.
- Clean up only on a later explicit request. Verify the merged PR, exact task
  head, clean worktree, refs, and task processes before deletion.

## Runtime and dependencies

- Linux with a working GPU window session. KDE Wayland uses XWayland.
- Rust 1.88 or later, edition 2024. Setup was checked with Rust/Cargo 1.96.0.
- Install CMake, a C compiler, Clang/libclang, and OpenGL/X11 development libraries
  through the system package manager. CMake 4.4.3 and Clang 22.1.8 were available
  at setup.
- Ensure the Rust `rustfmt` and `clippy` components are installed.
- Run `cargo fetch --locked` in each new worktree. Cargo uses the checked-in
  `Cargo.lock`; build output stays in that worktree's ignored `target/` directory.
- No project secrets or local environment file are required. Do not commit
  credentials, user settings, or saved results.
- Normal app runs use the user's XDG settings and state directories. Use the
  smoke test for isolated QA data. Output under `artifacts/` is ignored.

## Required local checks

Run from the task worktree:

```sh
cargo fmt --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --release --locked
./target/release/aim-trainer --smoke-test artifacts/native-qa
```

All commands passed at setup on commit `cf3d490`. Six unit tests and all six
native scenarios passed. These results are a baseline, not evidence for future
task changes. The smoke test needs a real GPU window session and checks
countdown, pause/resume, results, disk reload, and screenshots.

No GitHub Actions workflows or branch rules were present at setup. Check live
repository requirements for each task. Required CI must pass before readiness;
unavailable or pending checks do not count as passing. Squash merge was enabled.

## Herdr and review

- Use one Herdr workspace for this project. At setup it was `aim-trainer`, ID
  `wE`; the coordinator was in `wE:t1`, pane `wE:p1`. IDs can change. Read live
  state before use. Do not assume other existing tabs belong to this task.
- Check `HERDR_ENV=1` and read the Herdr skill before session access.
- Use separate task implementation and review tabs. Create task tabs with
  `herdr tab create --workspace <id> --cwd <worktree> --label <label> --no-focus`.
  Read returned IDs. Preserve the user's focus and user-owned sessions.
- Start the fresh reviewer in an available shell pane with:

  ```sh
  herdr agent start <name> --kind codex --pane <id> -- --sandbox read-only --ask-for-approval never --no-alt-screen -C <worktree>
  ```

- Give the reviewer the issue, base, commit, acceptance criteria, project
  instructions, and check results through `herdr agent prompt`. Pause edits
  during review. The reviewer must report actionable defects without edits.
- Read the actual report with
  `herdr agent read <name> --source recent-unwrapped --lines 120`.
  Agent lifecycle state alone is not review evidence. If the report is cut off,
  follow the Herdr skill's report recovery procedure.
- Fix findings, repeat affected checks, and request fresh review. Keep the PR
  draft until all criteria and required checks pass with no unresolved defects.
- Installed CLI help confirmed these options at setup. Read installed help
  again if syntax changes.
- Readiness notification syntax:
  `herdr notification show 'PR ready' --body '<PR URL>' --sound done`.
  The setup notification returned `shown: true`. If later delivery fails,
  report the limit in the Codex session. Do not change global settings.
