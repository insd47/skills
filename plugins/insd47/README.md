# insd47

Codex와 Claude Code가 동일한 스킬 본문을 사용하는 공용 스킬 플러그인이다. 사용자가 저장소를 직접 clone하거나 스킬 디렉터리에 심볼릭 링크를 만들 필요는 없다.

## 스킬

- [`build`](skills/build/SKILL.md) — 개인 구현 스타일의 핵심 구현 규칙. 언어별 규칙은 실제 작업 언어에 따라 선택적으로 로드한다.
  - [Rust](skills/build/languages/RUST.md)
  - [TypeScript 및 TSX](skills/build/languages/TYPESCRIPT.md)
- [`codex-app`](skills/codex-app/SKILL.md) — Claude가 계획·비평·조율을, Codex가 구현·조사를 맡는 협업 규칙. 하네스를 판별해 Claude 또는 Codex 가이드 하나만 로드한다. 구현처럼 맥락이 쌓여야 하는 작업은 persistent 스레드로, 조사는 병렬 일회용 워커로 라우팅한다.

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

공용 스킬은 `$insd47:build`와 `$insd47:codex-app`이다. 요청과 설명이 일치하면 에이전트가 자동으로 선택할 수도 있다.

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

공용 규칙은 `/insd47:build`와 `/insd47:codex-app`으로 호출한다.
