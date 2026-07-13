# skills

Personal implementation guidance for AI coding agents. [`build/SKILL.md`](build/SKILL.md) is the canonical entrypoint. Language-specific guidance lives under `build/languages/` and is loaded only when that language is in the current task.

## Skill source

Give an agent these two values:

```text
Repository: https://github.com/insd47/skills
Skill path: build
```

Ask it to install or update the `build` directory as a personal skill for the coding agents available on the machine. The repository is public so bootstrap agents can fetch it without GitHub credentials.

## Layout

```text
.
├── build/
│   ├── SKILL.md
│   ├── languages/
│   │   ├── RUST.md
│   │   └── TYPESCRIPT.md
│   └── agents/
│       └── openai.yaml
└── README.md
```

## Agent-managed installation

An agent can keep one checkout and expose its `build` directory to both Codex and Claude Code:

```sh
root="${XDG_DATA_HOME:-$HOME/.local/share}/insd-skills"

if git -C "$root" rev-parse --git-dir >/dev/null 2>&1; then
  git -C "$root" pull --ff-only
else
  git clone https://github.com/insd47/skills.git "$root"
fi

mkdir -p "${CODEX_HOME:-$HOME/.codex}/skills"
mkdir -p "$HOME/.claude/skills"

ln -sfn "$root/build" "${CODEX_HOME:-$HOME/.codex}/skills/build"
ln -sfn "$root/build" "$HOME/.claude/skills/build"
```

Start a new agent session after the first installation or after skill metadata changes.

## Invocation

Codex:

```text
$build Implement the requested change.
```

Claude Code:

```text
/build Implement the requested change.
```

Both agents may also select the skill automatically when a request matches its description.

## Verify installation

```sh
test -f "${CODEX_HOME:-$HOME/.codex}/skills/build/SKILL.md"
test -f "$HOME/.claude/skills/build/SKILL.md"
```

Both commands should exit successfully.
