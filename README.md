# skills

AI 코딩 에이전트에서 사용하는 개인 스킬과 Claude-Codex 협업 도구 모음이다. 스킬 본문은 에이전트의 해석 성능을 위해 영어로 작성하고, 설치 및 사용 문서는 한국어로 작성한다.

## 플러그인

| 플러그인                                   | 설명                                                               | 지원 제품          |
|--------------------------------------------|--------------------------------------------------------------------|--------------------|
| [`insd47`](plugins/insd47/README.md)       | 공용 구현 규칙(`build`)과 Claude-Codex 협업 규칙(`codex`)           | Codex, Claude Code |
| [`codex-app`](plugins/codex-app/README.md) | Codex App 작업을 Claude Code에서 조율하는 native MCP tools          | Claude Code        |

설치 방법과 상세 동작은 각 플러그인의 README에 있다. 두 플러그인 모두 이 저장소를 marketplace로 등록한 뒤 설치하며, 사용자가 저장소를 직접 clone하거나 스킬 디렉터리에 심볼릭 링크를 만들
필요는 없다.

제품마다 원격 플러그인을 선언하는 방식은 서로 다르다.

| 제품        | 원격 설치 방식                                | 이 저장소의 지원 상태 |
|-------------|-----------------------------------------------|-----------------------|
| Codex       | Git marketplace 등록 후 plugin 설치           | 지원                  |
| Claude Code | 설정 또는 CLI로 Git marketplace와 plugin 선언 | 지원                  |
| OpenCode    | `plugin` 배열의 NPM 실행 모듈                 | 별도 어댑터가 필요함  |

## 개발과 검증

플러그인 manifest의 버전을 변경하지 않으면 Claude Code가 기존 버전을 계속 사용할 수 있으므로 배포할 때 `version`을 함께 올린다.

`codex-app`은 Apple Silicon Mac의 Rust toolchain에서 검증한다.

```sh
cargo fmt --check --manifest-path plugins/codex-app/Cargo.toml
cargo test --manifest-path plugins/codex-app/Cargo.toml
cargo clippy --manifest-path plugins/codex-app/Cargo.toml --all-targets -- -D warnings
cargo build --release --manifest-path plugins/codex-app/Cargo.toml
claude plugin validate .
```
