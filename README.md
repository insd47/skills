# skills

AI 코딩 에이전트에서 사용하는 개인 스킬과 Claude-Codex 협업 도구 모음입니다.

스킬 본문은 에이전트가 정확하게 해석하도록 영어로 작성하고, 설치 및 사용 문서는 한국어로 작성합니다. 한국어 예시가 중심인 `KOREAN.md`만 예외적으로 한국어로 작성합니다.

## 플러그인

| 플러그인                                   | 설명                                                                  | 지원 제품          |
|--------------------------------------------|-----------------------------------------------------------------------|--------------------|
| [`insd47`](plugins/insd47/README.md)       | 개인 개발 원칙과 언어별 구현 규칙(`build`), 문서·주석·커밋 작성 규칙(`write`) | Codex, Claude Code |
| [`codex-app`](plugins/codex-app/README.md) | Codex App 작업을 Claude Code에서 조율하는 native MCP tools             | Claude Code        |

설치 방법과 상세 동작은 각 플러그인의 README를 참조하세요. 두 플러그인 모두 이 저장소를 marketplace로 등록한 후 설치하므로, 저장소를 직접 clone하거나 스킬 디렉터리에 심볼릭 링크를 만들 필요는 없습니다.

원격 플러그인을 선언하는 방식은 제품마다 다릅니다.

| 제품        | 원격 설치 방식                                | 이 저장소의 지원 상태 |
|-------------|-----------------------------------------------|-----------------------|
| Codex       | Git marketplace 등록 후 plugin 설치           | 지원                  |
| Claude Code | 설정 또는 CLI로 Git marketplace와 plugin 선언 | 지원                  |
| OpenCode    | `plugin` 배열의 NPM 실행 모듈                 | 별도 어댑터가 필요함  |

## 개발과 검증

> 배포할 때는 플러그인 manifest의 `version`을 함께 올려 주세요. 버전이 그대로면 Claude Code가 기존 버전을 계속 사용할 수 있습니다.

`codex-app`은 Apple Silicon Mac의 Rust toolchain에서 검증합니다.

```sh
cargo fmt --check --manifest-path plugins/codex-app/Cargo.toml
cargo test --manifest-path plugins/codex-app/Cargo.toml
cargo clippy --manifest-path plugins/codex-app/Cargo.toml --all-targets -- -D warnings
cargo build --release --manifest-path plugins/codex-app/Cargo.toml
claude plugin validate .
```
