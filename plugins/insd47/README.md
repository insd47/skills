# insd47

Codex와 Claude Code가 동일한 스킬 본문을 사용하는 공용 스킬 플러그인이다. 사용자가 저장소를 직접 clone하거나 스킬 디렉터리에 심볼릭 링크를 만들 필요는 없다.

## 스킬

- [`build`](skills/build/SKILL.md) — 개인 구현 스타일의 핵심 구현 규칙. 언어별 규칙은 실제 작업 언어에 따라 선택적으로 로드한다.
  - [Rust](skills/build/languages/RUST.md)
  - [TypeScript 및 TSX](skills/build/languages/TYPESCRIPT.md)
- [`codex`](skills/codex/SKILL.md) — Claude가 계획·비평·조율을, Codex가 구현·조사를 맡는 협업 규칙. 구현은 `codex-app` 플러그인의 `ask` MCP tool로 persistent task에 위임하고, 조사는 `codex` CLI를 background task로 직접 실행한다.

## Codex 설치

저장소를 marketplace로 등록하고 플러그인을 설치한다.

```sh
codex plugin marketplace add insd47/skills
codex plugin add insd47@insd47-skills
```

업데이트는 marketplace snapshot을 갱신한 뒤 플러그인을 다시 설치하는 방식으로 적용한다.

```sh
codex plugin marketplace upgrade insd47-skills
codex plugin add insd47@insd47-skills
```

공용 스킬은 `$insd47:build`와 `$insd47:codex`다. 요청과 설명이 일치하면 에이전트가 자동으로 선택할 수도 있다.

## Claude Code 설치

marketplace를 등록하고 플러그인을 설치한 뒤 `/reload-plugins`로 현재 세션에 반영한다.

```sh
claude plugin marketplace add insd47/skills
claude plugin install insd47@insd47-skills
```

업데이트는 marketplace snapshot을 갱신한 뒤 플러그인을 최신 버전으로 올리는 방식으로 적용하며, 세션을 다시 시작하거나 `/reload-plugins`를 실행해야 반영된다.

```sh
claude plugin marketplace update insd47-skills
claude plugin update insd47@insd47-skills
```

공용 규칙은 `/insd47:build`와 `/insd47:codex`로 호출한다. `codex` 협업 규칙은 구현 위임에 별도 실행 플러그인(`codex-app`)의 native MCP tools를, 조사 위임에 `codex` CLI를 사용한다.
