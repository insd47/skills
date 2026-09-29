# insd47

Codex와 Claude Code가 같은 스킬 본문을 사용하는 공용 스킬 플러그인입니다.

## 스킬

- [`build`](skills/build/SKILL.md): 개인 구현 스타일의 핵심 구현 규칙입니다. 언어별 규칙은 실제 작업 언어에 따라 선택적으로 로드합니다.
  - [Rust](skills/build/languages/RUST.md)
  - [TypeScript 및 TSX](skills/build/languages/TYPESCRIPT.md)
- [`write`](skills/write/SKILL.md): README와 `.docs` 문서, 문서 주석과 코드 주석, 오류 메시지, 커밋과 PR 메시지의 작성 규칙입니다.
  - [한국어 문장 지침](skills/write/KOREAN.md): [snflkd/fluent-korean](https://github.com/snflkd/fluent-korean)(MIT)의 output style 본문을 그대로 옮기고, 원본 README의 세부 동작 두 가지를 덧붙였습니다. 사용자 말투 기준은 `write` 본문에 있습니다.

스킬 본문은 주력 모델인 Claude Opus 5.5의 [프롬프팅 가이드](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/prompting-claude-opus-5-5)에 맞춰 작업 범위와 검증·보고 지시를 줄였습니다. 모델이 스스로 수행하는 재검증과 범위 확장을 부추기는 문장은 두지 않으며, Codex에서도 같은 본문을 사용합니다.

### 적용 순서

두 스킬은 사용자의 직접 지시와 레포의 AGENTS.md 또는 CLAUDE.md를 가장 먼저 따릅니다. 그다음에는 레포를 누가 관리하는지에 따라 적용 방식이 달라집니다.

- 사용자가 주 작성자인 레포나 관례가 아직 없는 신생 레포에서는 스킬을 그 레포의 관례로 적용합니다.
- 다른 사람이 관리하는 레포에서는 그 레포의 관례를 따르며, 스킬은 코드베이스가 정하지 않은 결정에만 사용합니다.

## Codex 설치

저장소를 marketplace로 등록한 후 플러그인을 설치합니다.

```sh
codex plugin marketplace add insd47/skills
codex plugin add insd47@insd47-skills
```

업데이트하려면 marketplace snapshot을 갱신한 후 플러그인을 다시 설치합니다.

```sh
codex plugin marketplace upgrade insd47-skills
codex plugin add insd47@insd47-skills
```

스킬은 `$insd47:build`와 `$insd47:write`로 호출합니다. 요청이 스킬 설명과 일치하면 에이전트가 자동으로 선택할 수도 있습니다.

## Claude Code 설치

marketplace를 등록한 후 플러그인을 설치하고, `/reload-plugins`로 현재 세션에 반영합니다.

```sh
claude plugin marketplace add insd47/skills
claude plugin install insd47@insd47-skills
```

업데이트하려면 marketplace snapshot을 갱신한 후 플러그인을 최신 버전으로 올립니다.

```sh
claude plugin marketplace update insd47-skills
claude plugin update insd47@insd47-skills
```

> 업데이트는 세션을 다시 시작하거나 `/reload-plugins`를 실행해야 반영됩니다.

스킬은 `/insd47:build`와 `/insd47:write`로 호출합니다.
