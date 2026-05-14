# Test Fixtures

Pre-built Git repositories used as canonical test data for all parser, engine, and integration tests.

## Conventions

- Each fixture is a minimal Git repository (`git init` + initial commit)
- Claude Code config files live under `.claude/` within the fixture
- Fixture names use kebab-case: `repo-<descriptor>`
- Keep fixtures minimal: only include files needed for the test scenario
- When parser behavior changes, update affected fixtures and re-run golden file tests

## Fixture Catalog

| Fixture | Description |
|----------|------------|
| `repo-basic` | Single skill + basic `.claude/settings.json` |
| `repo-full` | All resource types (skills, MCP, hooks, rules, agents) |
| `repo-broken` | Malformed JSON, missing files, broken references |
| `repo-conflict` | Overlapping resource names for migration testing |

## Creating a New Fixture

```bash
mkdir tests/fixtures/repo-<name>
cd tests/fixtures/repo-<name>
git init
git add .
git commit -m "initial fixture"
```
