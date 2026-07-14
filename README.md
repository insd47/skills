# skills

AI 코딩 에이전트에서 사용하는 개인 스킬 모음이다. 현재 `insd47` 플러그인의 `build` 스킬은 계층적 모듈, 제한된 공개 범위, 단순한 재귀적 조립, 구조적 대칭성, 선택적인 타입 선언과 상황에 맞는 오류 처리를 중심으로 구현 작업을 안내한다.

스킬 본문은 에이전트의 해석 성능을 위해 영어로 작성하고, 설치 및 사용 문서는 한국어로 작성한다.

## 저장소 구조

```text
.
├── .agents/plugins/marketplace.json
├── .claude-plugin/marketplace.json
└── plugins/insd47/
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
codex plugin add insd47@insd47-skills
```

두 명령 모두 Codex가 GitHub 저장소를 자체 cache로 가져오므로 사용자가 clone할 필요가 없다. bootstrap을 맡은 에이전트에는 위 두 명령을 실행하도록 요청하면 된다.

기존 `build@insd-skills` 설치는 이름이 달라 자동으로 이전되지 않으므로 한 번 제거한 뒤 새 이름으로 다시 등록한다.

```sh
codex plugin remove build@insd-skills
codex plugin marketplace remove insd-skills
codex plugin marketplace add insd47/skills
codex plugin add insd47@insd47-skills
```

현재 Codex CLI에서는 `config.toml`에 marketplace URL과 plugin ID만 직접 적은 깨끗한 환경에서 marketplace snapshot을 내려받지 않는다. 따라서 선언만 복사하는 방식보다 `marketplace add`를 공식 bootstrap 단계로 사용해야 한다. marketplace의 `INSTALLED_BY_DEFAULT` 정책도 선언했지만, 새 CLI 환경에서는 명시적인 `plugin add`까지 실행하는 것이 확실하다.

업데이트는 marketplace snapshot을 갱신한 뒤 플러그인을 다시 설치하는 방식으로 적용한다.

```sh
codex plugin marketplace upgrade insd47-skills
codex plugin add insd47@insd47-skills
```

호출 이름은 플러그인 namespace가 적용된 `$insd47:build`다. 구현 요청과 설명이 일치하면 에이전트가 자동으로 선택할 수도 있다.

## Claude Code

marketplace를 등록하고 플러그인을 설치한다.

```sh
claude plugin marketplace add insd47/skills
claude plugin install insd47@insd47-skills
```

기존 설치에서 이전할 때도 이전 플러그인과 marketplace를 한 번 제거한 뒤 위 명령으로 다시 등록한다.

```sh
claude plugin uninstall build@insd-skills
claude plugin marketplace remove insd-skills
claude plugin marketplace add insd47/skills
claude plugin install insd47@insd47-skills
```

설치 후 현재 세션에 반영한다.

```text
/reload-plugins
```

호출 이름은 `/insd47:build`다. Claude Code 플러그인의 스킬 이름은 충돌 방지를 위해 항상 plugin namespace를 포함한다.

Claude Code 설정에 marketplace와 플러그인을 선언할 수도 있다.

```json
{
  "extraKnownMarketplaces": {
    "insd47-skills": {
      "source": {
        "source": "github",
        "repo": "insd47/skills"
      }
    }
  },
  "enabledPlugins": {
    "insd47@insd47-skills": true
  }
}
```

이 설정을 사용자 또는 프로젝트 설정에 넣으면 저장소를 직접 clone하지 않고 marketplace와 플러그인을 가져올 수 있다.

## 스킬 개발

핵심 구현 규칙은 [`plugins/insd47/skills/build/SKILL.md`](plugins/insd47/skills/build/SKILL.md)에 있다. 언어별 규칙은 실제 작업 언어에 따라 선택적으로 로드한다.

- [Rust](plugins/insd47/skills/build/languages/RUST.md)
- [TypeScript 및 TSX](plugins/insd47/skills/build/languages/TYPESCRIPT.md)

플러그인 manifest의 버전을 변경하지 않으면 Claude Code가 기존 버전을 계속 사용할 수 있으므로 배포할 때 `version`을 함께 올린다.
