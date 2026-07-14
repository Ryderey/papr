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
