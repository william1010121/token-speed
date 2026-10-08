# Codex review

This repository includes `AGENTS.md` with project-specific **Code Review Rules** covering measurement integrity, local-data boundaries and interactive reliability. Deterministic formatting, lint and test checks run in GitHub Actions.

## GitHub integration

Automatic reviews are an account / repository setting in Codex, not a switch in a committed TOML file:

1. Connect `william1010121/token-speed` to Codex, granting the GitHub integration access to this repository.
2. Open [Codex code review settings](https://chatgpt.com/codex/settings/code-review), select this repository, and enable automatic review for all pull requests.
3. Choose the review trigger offered by the settings UI. Reviewing when a PR opens and after subsequent pushes keeps feedback current.

A maintainer can request a one-off review with a PR comment:

```text
@codex review
```

The repository instructions guide findings; CI remains responsible for mechanical checks. Do not add a required Codex status check until the integration actually reports one. No `OPENAI_API_KEY` GitHub secret is required for the native Codex GitHub integration.

For a local check, use the installed Codex CLI:

```bash
codex review --uncommitted
codex review --base main
```

See [OpenAI's GitHub review documentation](https://developers.openai.com/codex/integrations/github/) for supported settings and current behavior. Enabling review consumes the reviewing account's applicable Codex allowance; this project does not bundle an API-powered review workflow.
