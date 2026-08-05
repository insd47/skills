# codex-app

Codex 작업을 Claude Code의 native MCP tools로 위임하고 이어서 조율하는 Claude 전용 플러그인이다. persistent task의 맥락 위에 쌓이는 구현과 후속 작업을 `ask`로 전달하고, 같은 turn을 조회하거나 steer·interrupt한다. MCP server instructions가 도구 선택과 조율 규칙을 제공하며, 플러그인 자체는 별도 Skill이나 Bash 호출을 로드하지 않는다.

> [!IMPORTANT]
> macOS 전용이다. macOS용 ChatGPT Desktop이 소유한 Codex task와 Desktop IPC로 통신한다.

## 설치

marketplace를 등록하고 플러그인을 설치한다. 설치되거나 업데이트된 MCP server는 `/reload-plugins`로 현재 세션에 반영할 수 있다. 단, `.mcp.json`의 tool timeout은 세션 시작 시 로드되므로 이 설정까지 바뀌었다면 Claude Code 세션을 새로 연다.

Apple Silicon Mac과 Claude Code 2.1.212 이상이 필요하다. 플러그인은 배포된 단일 실행 파일을 사용하며 Node.js를 요구하지 않는다. Claude Code 최소 버전은 2분을 넘긴 MCP tool call을 자동으로 background 전환하는 기준이다.

```sh
claude plugin marketplace add insd47/skills
claude plugin install codex-app@insd47-skills
```

업데이트는 marketplace snapshot을 갱신한 뒤 플러그인을 최신 버전으로 올리는 방식으로 적용한다. 보통 `/reload-plugins`면 충분하며, `.mcp.json`의 timeout 변경까지 적용하려면 Claude Code 세션을 다시 시작한다.

```sh
claude plugin marketplace update insd47-skills
claude plugin update codex-app@insd47-skills
```

## MCP tools

```text
ask(prompt)
status(threadId, turnId)
steer(threadId, turnId, prompt)
interrupt(threadId, turnId)
```

도구는 플러그인을 활성화하면 자동으로 연결되며 `/mcp`에서 `codex-app` server와 tool 목록을 확인할 수 있다. `alwaysLoad`를 사용하므로 별도 Tool Search 없이 자연어 요청에서 바로 선택할 수 있다.

## 동작

`ask` MCP tool은 Codex turn이 끝날 때까지 block한 뒤 `{threadId, turnId, status, result}`를 최종 tool result로 반환한다. `status`와 `result`는 Desktop follower의 snapshot과 이어지는 patch stream에서 판정한 terminal 상태와 마지막 assistant message다. 2분을 넘긴 호출은 Claude Code 2.1.212 이상에서 자동으로 background 전환되고, turn이 끝나면 task notification으로 같은 tool result가 전달된다. plugin 고유 job ID나 영속 job registry를 만들지 않으며 별도 polling도 필요 없다.

실행 중인 turn이 있을 때 `ask`를 다시 호출하면 같은 persistent 작업을 자동으로 steer하고 `{status: "steered", threadId, turnId}`를 즉시 반환한다. 최초의 blocked `ask` 하나만 turn의 completion을 기다린다. `status`, `steer`, `interrupt`는 `ask`가 보고한 Codex `threadId`와 `turnId`를 그대로 사용한다. 별도 상태 목록은 제공하지 않으며 `status`는 해당 turn이 현재 MCP server 세션의 메모리 레지스트리에 있는지만 확인한다. `interrupt`는 non-blocking으로 Codex turn만 정상 중단하고 task는 보존하며, follower가 중단 상태를 보내면 blocked `ask`가 `interrupted`로 끝난다. 서버 재시작 전후로 제어 상태를 이어 붙이지 않는다.

`ask`는 현재 ChatGPT Desktop이 소유한 task에 `${CODEX_HOME:-$HOME/.codex}/ipc/ipc.sock`을 통해 turn을 전달하므로 프롬프트와 진행 상황이 Desktop에 바로 표시된다. 시작 전에 Desktop의 task-state follower로 등록하고, 최초 snapshot과 revision이 이어지는 patch stream에서만 진행 및 완료 상태를 판정한다. 사용자의 중단, 플러그인의 중단, 정상 완료가 모두 Desktop이 실제로 표시하는 상태와 같은 출처에서 전달되며 별도 App Server의 디스크 재구성 상태나 무응답 시간은 완료 근거로 사용하지 않는다.

ChatGPT Desktop은 내장 터미널을 만들 때 대화 제목을 `CODEX_APP_TITLE`로 전달하며, 플러그인은 현재 작업 디렉터리와 정확히 일치하는 비아카이브 task 중에서 제목까지 정확히 일치하는 항목이 하나일 때만 자동 선택한다. App Server의 `thread/list`에 `cwd`와 `archived: false`를 함께 전달하므로 다른 프로젝트의 동명 task는 후보로 로드되지 않는다. 이 제목 조회에만 짧게 실행한 App Server를 사용하고 task를 찾은 즉시 닫는다. 이후 turn 시작, steer, interrupt, 상태 stream, 최종 메시지는 모두 Desktop IPC를 사용한다. 제목이 없거나 현재 작업 디렉터리 안에서도 일치 결과가 모호하면 task ID를 요구한다. 명시적으로 고정하려면 Claude를 시작할 때 `CODEX_APP_THREAD_ID=<id>`를 설정하며, 이 방식은 프로젝트 경로와 아카이브 여부에 관계없이 전역으로 해당 ID를 사용한다.

Codex나 Claude는 이미 실행 중인 사람용 내장 터미널 프로세스에 환경 변수를 사후 주입할 수 없다. `CODEX_APP_TITLE`은 터미널 생성 시점의 스냅샷이므로 task 제목을 바꾼 뒤에는 터미널을 다시 열거나 `CODEX_APP_THREAD_ID`를 사용한다.

Desktop IPC는 현재 앱이 창간 task coordination에 사용하는 비공개 프로토콜이다. 플러그인은 앱과 동일한 길이 프레이밍, follower broadcast, snapshot revision 및 patch 규칙을 사용하고, socket 소유권과 디렉터리 권한을 검사한다. patch revision이 끊기면 IPC에서 새 snapshot을 요청한다. 앱 업데이트로 프로토콜 버전이나 상태 형식이 달라지면 별도 App Server task나 rollout polling으로 조용히 우회하지 않고 오류를 반환한다. `CODEX_APP_IPC_PATH`로 endpoint를 명시할 수 있지만 일반 설치에서는 canonical 경로를 그대로 사용한다.

병렬 또는 일회용 research probe는 이 플러그인의 책임이 아니며 `insd47:codex` Skill의 안내에 따라 `codex exec`를 직접 사용한다.

## 개발

MCP server는 `src/`의 Rust crate에서 빌드하며 `server` 실행 mode만 제공한다. stdio MCP에는 공식 `rmcp` SDK를 사용하고 wire type은 Serde로 정의한다.

`src/client/mod.rs`는 Desktop IPC facade, `transport.rs`는 endpoint와 framing, `follower.rs`는 task-state stream, `lookup.rs`는 제목 조회용 App Server 수명을 소유한다. `src/turns/`는 facade, persistent task, registry로 turn 수명을 관리한다.

```sh
cargo fmt --check --manifest-path plugins/codex-app/Cargo.toml
cargo test --manifest-path plugins/codex-app/Cargo.toml
cargo clippy --manifest-path plugins/codex-app/Cargo.toml --all-targets -- -D warnings
cargo build --release --manifest-path plugins/codex-app/Cargo.toml
claude plugin validate .
```

릴리스할 때는 Apple Silicon Mac에서 release binary를 빌드한 뒤 `plugins/codex-app/bin/codex-app`으로 복사하고 실행 권한을 `755`로 맞춘다. 소스, release binary, `plugins/codex-app/.claude-plugin/plugin.json`, marketplace의 버전을 함께 올린다.
