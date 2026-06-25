# Issue tracker: GitHub

Issues and PRDs for this repo live as GitHub issues. Use the `gh` CLI for all operations.

## Conventions

- Create: `gh issue create --title "..." --body "..."`
- Read: `gh issue view <number> --comments`
- List: `gh issue list` with appropriate label and state filters
- Comment: `gh issue comment <number> --body "..."`
- Add/remove labels: `gh issue edit <number> --add-label "..."` or `--remove-label "..."`
- Close: `gh issue close <number> --comment "..."`

Run commands inside the repository so `gh` infers `Ryderey/papr` from the Git remote.

## Pull requests as a triage surface

PRs as a request surface: **no**.

External pull requests do not enter the issue triage workflow.

## Skill operations

When a skill says “publish to the issue tracker,” create a GitHub issue.

When a skill says “fetch the relevant ticket,” run:

`gh issue view <number> --comments`
