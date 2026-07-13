# skills

AI 코딩 에이전트에서 사용하는 개인 스킬 모음이다. 현재 `build` 플러그인은 계층적 모듈, 제한된 공개 범위, 단순한 재귀적 조립, 구조적 대칭성, 선택적인 타입 선언과 상황에 맞는 오류 처리를 중심으로 구현 작업을 안내한다.

스킬 본문은 에이전트의 해석 성능을 위해 영어로 작성하고, 설치 및 사용 문서는 한국어로 작성한다.

## 저장소 구조

```text
.
├── .agents/plugins/marketplace.json
├── .claude-plugin/marketplace.json
└── plugins/build/
    ├── .codex-plugin/plugin.json
    ├── .claude-plugin/plugin.json
    └── skills/build/
        ├── SKILL.md
        ├── languages/
        │   ├── RUST.md
        │   └── TYPESCRIPT.md
        └── agents/openai.yaml
```

Codex와 Claude Code가 동일한 플러그인과 스킬 본문을 사용한다. 사용자가 저장소를 직접 clone하거나 스킬 디렉터리에 심볼릭 링크를 만들 필요는 없다.

제품마다 원격 플러그인을 선언하는 방식은 서로 다르다.

| 제품 | 원격 설치 방식 | 이 저장소의 지원 상태 |
| --- | --- | --- |
| Codex | Git marketplace 등록 후 plugin 설치 | 지원 |
| Claude Code | 설정 또는 CLI로 Git marketplace와 plugin 선언 | 지원 |
| OpenCode | `plugin` 배열의 NPM 실행 모듈 | 별도 어댑터가 필요함 |

## Codex

저장소를 marketplace로 등록하고 플러그인을 설치한다.

```sh
codex plugin marketplace add insd47/skills
codex plugin add build@insd-skills
```

두 명령 모두 Codex가 GitHub 저장소를 자체 cache로 가져오므로 사용자가 clone할 필요가 없다. bootstrap을 맡은 에이전트에는 위 두 명령을 실행하도록 요청하면 된다.

현재 Codex CLI에서는 `config.toml`에 marketplace URL과 plugin ID만 직접 적은 깨끗한 환경에서 marketplace snapshot을 내려받지 않는다. 따라서 선언만 복사하는 방식보다 `marketplace add`를 공식 bootstrap 단계로 사용해야 한다. marketplace의 `INSTALLED_BY_DEFAULT` 정책도 선언했지만, 새 CLI 환경에서는 명시적인 `plugin add`까지 실행하는 것이 확실하다.

업데이트는 marketplace snapshot을 갱신한 뒤 플러그인을 다시 설치하는 방식으로 적용한다.

```sh
codex plugin marketplace upgrade insd-skills
codex plugin add build@insd-skills
```

호출 이름은 플러그인 namespace가 적용된 `$build:build`다. 구현 요청과 설명이 일치하면 에이전트가 자동으로 선택할 수도 있다.

## Claude Code

marketplace를 등록하고 플러그인을 설치한다.

```sh
claude plugin marketplace add insd47/skills
claude plugin install build@insd-skills
```

설치 후 현재 세션에 반영한다.

```text
/reload-plugins
```

호출 이름은 `/build:build`다. Claude Code 플러그인의 스킬 이름은 충돌 방지를 위해 항상 plugin namespace를 포함한다.

Claude Code 설정에 marketplace와 플러그인을 선언할 수도 있다.

```json
{
  "extraKnownMarketplaces": {
    "insd-skills": {
      "source": {
        "source": "github",
        "repo": "insd47/skills"
      }
    }
  },
  "enabledPlugins": {
    "build@insd-skills": true
  }
}
```

이 설정을 사용자 또는 프로젝트 설정에 넣으면 저장소를 직접 clone하지 않고 marketplace와 플러그인을 가져올 수 있다.

## OpenCode

OpenCode의 `plugin` 배열은 Git marketplace나 Agent Skill 패키지를 설치하는 기능이 아니다. 배열에 지정한 NPM 패키지를 Bun으로 설치한 뒤 JavaScript 또는 TypeScript hook으로 실행한다.

```json
{
  "$schema": "https://opencode.ai/config.json",
  "plugin": ["@insd47/opencode-build"]
}
```

따라서 위와 같은 설정만으로 `build` 스킬을 제공하려면 별도의 NPM 플러그인 어댑터가 필요하다. 어댑터는 다음 중 하나를 수행해야 한다.

1. 패키지에 포함한 `SKILL.md`를 OpenCode의 전역 스킬 디렉터리에 동기화한다.
2. OpenCode plugin API로 별도의 tool이나 hook을 제공해 스킬 내용을 동적으로 주입한다.

첫 번째 방식은 기존 Agent Skill과 가장 잘 호환되지만, 플러그인이 시작할 때 사용자 설정 디렉터리에 파일을 쓰는 부작용이 있다. 두 번째 방식은 파일을 설치하지 않지만 OpenCode 전용 구현이 되고 다른 에이전트와 동일한 `skill` 호출 경험을 제공하지 못한다.

현재 저장소는 Codex와 Claude Code의 공식 marketplace 방식까지만 제공한다. OpenCode의 `plugin` 배열과 동일한 설치 경험이 반드시 필요할 때만 `@insd47/opencode-build` NPM 어댑터를 별도로 추가하는 것이 적절하다. 단순한 문서 배포를 위해 사용자 설정 디렉터리에 파일을 쓰는 NPM 패키지를 지금 추가하는 것은 유지보수 비용과 부작용에 비해 이점이 작다.

## 스킬 개발

핵심 구현 규칙은 [`plugins/build/skills/build/SKILL.md`](plugins/build/skills/build/SKILL.md)에 있다. 언어별 규칙은 실제 작업 언어에 따라 선택적으로 로드한다.

- [Rust](plugins/build/skills/build/languages/RUST.md)
- [TypeScript 및 TSX](plugins/build/skills/build/languages/TYPESCRIPT.md)

플러그인 manifest의 버전을 변경하지 않으면 Claude Code가 기존 버전을 계속 사용할 수 있으므로 배포할 때 `version`을 함께 올린다.
