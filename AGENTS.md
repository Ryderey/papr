# AGENTS.md

## Basic Working Rules

- Read relevant code before modifying.
- Provide a brief plan before modifying.
- Only modify files directly related to the current task.
- Do not perform unrelated refactoring.
- Do not add new dependencies arbitrarily.
- Do not delete existing features unless explicitly required by the task.
- Maintain existing code style, naming conventions, and project structure.
- Prefer PowerShell commands in Windows environments.
- After completing modifications, you must specify the modified files, the reasons for the changes, and the verification method.

## Security Rules

- Do not execute high-risk commands such as delete, clear, overwrite, or reset unless explicitly requested by the user.
- Do not modify environment variables, secrets, tokens, or account configurations unless explicitly required by the task.
- Do not write secrets, tokens, or private configurations into code or logs.

## Dependency Rules

- Prefer using existing project dependencies.
- Before adding a new dependency, you must explain:
  - Why it is needed
  - Whether there are existing alternatives
  - Which files will be affected
- Do not introduce a large dependency for a minor feature.

## Testing and Verification

- After modifications, prioritize running the project's existing check commands.
- If unable to run tests or builds, you must explain the reason.
- Do not claim to have tested when no testing was actually performed.
- If only static checks were performed, state this explicitly.
- The current project does not target mobile platforms; related tests can be skipped.

## Output Format

After completing each task, summarize in the following format:

1. Root cause of the issue
2. Modifications made
3. Modified files
4. Verification results
5. Risks and follow-up recommendations

## Agent skills

### Issue tracker

Issues are tracked in GitHub Issues; external PRs are not a triage surface. See `docs/agents/issue-tracker.md`.

### Triage labels

The repository uses the five default triage label names. See `docs/agents/triage-labels.md`.

### Domain docs

This is a single-context repository. See `docs/agents/domain.md`.

<!-- TRELLIS:START -->
# Trellis Instructions

These instructions are for AI assistants working in this project.

This project is managed by Trellis. The working knowledge you need lives under `.trellis/`:

- `.trellis/workflow.md` — development phases, when to create tasks, skill routing
- `.trellis/spec/` — package- and layer-scoped coding guidelines (read before writing code in a given layer)
- `.trellis/workspace/` — per-developer journals and session traces
- `.trellis/tasks/` — active and archived tasks (PRDs, research, jsonl context)

If a Trellis command is available on your platform (e.g. `/trellis:finish-work`, `/trellis:continue`), prefer it over manual steps. Not every platform exposes every command.

If you're using Codex or another agent-capable tool, additional project-scoped helpers may live in:
- `.agents/skills/` — reusable Trellis skills
- `.codex/agents/` — optional custom subagents

Managed by Trellis. Edits outside this block are preserved; edits inside may be overwritten by a future `trellis update`.

<!-- TRELLIS:END -->
