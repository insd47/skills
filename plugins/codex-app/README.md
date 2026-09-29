# codex-app

Codex 작업을 Claude Code의 native MCP tools로 위임하고 이어서 조율하는 Claude 전용 플러그인입니다. persistent task의 맥락 위에 쌓이는 구현과 후속 작업을 `ask`로 전달하고, 같은 turn을 조회하거나 steer·interrupt합니다. 도구 선택과 조율 규칙은 MCP server instructions가 제공하며, 플러그인은 별도의 Skill이나 Bash 호출을 로드하지 않습니다.

> [!IMPORTANT]
> macOS 전용입니다. macOS용 ChatGPT Desktop이 소유한 Codex task와 Desktop IPC로 통신합니다.

## 설치

Apple Silicon Mac과 Claude Code 2.1.212 이상이 필요합니다. Claude Code의 최소 버전은 2분을 넘긴 MCP tool call을 자동으로 background로 전환하는 기능을 기준으로 정했습니다. 플러그인은 배포된 단일 실행 파일을 사용하므로 Node.js가 필요하지 않습니다.

marketplace를 등록한 후 플러그인을 설치합니다.

```sh
claude plugin marketplace add insd47/skills
claude plugin install codex-app@insd47-skills
```

업데이트하려면 marketplace snapshot을 갱신한 후 플러그인을 최신 버전으로 올립니다.

```sh
claude plugin marketplace update insd47-skills
claude plugin update codex-app@insd47-skills
```

> 설치하거나 업데이트한 MCP server는 `/reload-plugins`로 현재 세션에 반영됩니다. 다만 `.mcp.json`의 tool timeout은 세션을 시작할 때 로드되므로, 이 설정이 바뀌었다면 Claude Code 세션을 새로 시작해 주세요.

## MCP tools

```text
ask(prompt)
status(threadId, turnId)
steer(threadId, turnId, prompt)
interrupt(threadId, turnId)
```

플러그인을 활성화하면 도구가 자동으로 연결되며, `/mcp`에서 `codex-app` server와 tool 목록을 확인할 수 있습니다. `alwaysLoad`를 사용하므로 별도의 Tool Search 없이 자연어 요청에서 바로 선택됩니다.

## 동작

### 호출과 완료

`ask`는 Codex turn이 끝날 때까지 block한 후 `{threadId, turnId, status, result}`를 최종 tool result로 반환합니다. `status`와 `result`는 Desktop follower의 snapshot과 이어지는 patch stream에서 판정한 terminal 상태와 마지막 assistant message입니다.

2분을 넘긴 호출은 Claude Code 2.1.212 이상에서 자동으로 background로 전환되며, turn이 끝나면 같은 tool result가 task notification으로 전달됩니다. 따라서 플러그인은 고유한 job ID나 영속적인 job registry를 만들지 않으며, 별도의 polling도 필요하지 않습니다.

### steer와 interrupt

- 실행 중인 turn이 있을 때 `ask`를 다시 호출하면 같은 persistent 작업을 자동으로 steer하고, `{status: "steered", threadId, turnId}`를 즉시 반환합니다. turn의 완료를 기다리는 것은 처음 block된 `ask` 하나뿐입니다.
- `status`, `steer`, `interrupt`는 `ask`가 보고한 Codex `threadId`와 `turnId`를 그대로 사용합니다.
- `status`는 해당 turn이 현재 MCP server 세션의 메모리 레지스트리에 있는지만 확인하며, 별도의 상태 목록은 제공하지 않습니다.
- `interrupt`는 block하지 않고 Codex turn만 정상적으로 중단하며, task는 보존합니다. follower가 중단 상태를 보내면 block된 `ask`가 `interrupted`로 끝납니다.
- 서버를 재시작하면 이전의 제어 상태는 이어지지 않습니다.

### Desktop IPC

`ask`는 현재 ChatGPT Desktop이 소유한 task에 `${CODEX_HOME:-$HOME/.codex}/ipc/ipc.sock`을 통해 turn을 전달합니다. 그래서 프롬프트와 진행 상황이 Desktop에 바로 표시됩니다.

진행 및 완료 상태는 Desktop의 task-state follower로 등록한 후 받은 최초 snapshot과, revision이 이어지는 patch stream에서만 판정합니다. 사용자의 중단, 플러그인의 중단, 정상 완료가 모두 Desktop이 실제로 표시하는 상태와 같은 출처에서 전달되기 때문입니다. 별도 App Server가 디스크에서 재구성한 상태나 무응답 시간은 완료의 근거로 사용하지 않습니다.

Desktop IPC는 앱이 창 사이의 task 조율에 사용하는 비공개 프로토콜입니다. 플러그인은 앱과 같은 길이 프레이밍, follower broadcast, snapshot revision 및 patch 규칙을 사용하고, socket의 소유권과 디렉터리 권한을 검사합니다. patch revision이 끊기면 IPC로 새 snapshot을 요청합니다.

> 앱 업데이트로 프로토콜 버전이나 상태 형식이 달라지면, 별도 App Server task나 rollout polling으로 조용히 우회하지 않고 오류를 반환합니다. `CODEX_APP_IPC_PATH`로 endpoint를 지정할 수 있지만, 일반적인 설치에서는 기본 경로를 그대로 사용해 주세요.

### task 선택

ChatGPT Desktop은 내장 터미널을 만들 때 대화 제목을 `CODEX_APP_TITLE`로 전달합니다. 플러그인은 현재 작업 디렉터리와 정확히 일치하는 아카이브되지 않은 task 중에서, 제목까지 정확히 일치하는 task가 하나일 때만 자동으로 선택합니다.

- App Server의 `thread/list`에 `cwd`와 `archived: false`를 함께 전달하므로, 다른 프로젝트의 같은 이름 task는 후보에 포함되지 않습니다.
- App Server는 이 제목 조회에만 잠시 실행하고, task를 찾으면 즉시 종료합니다. 이후의 turn 시작, steer, interrupt, 상태 stream, 최종 메시지는 모두 Desktop IPC를 사용합니다.
- 제목이 없거나, 현재 작업 디렉터리 안에서도 일치하는 task가 여럿이면 task ID를 요구합니다.
- task를 명시적으로 고정하려면 Claude를 시작할 때 `CODEX_APP_THREAD_ID=<id>`를 설정합니다. 이 방식은 프로젝트 경로와 아카이브 여부와 관계없이 해당 ID를 사용합니다.

> `CODEX_APP_TITLE`은 터미널을 만든 시점의 값이며, 이미 실행 중인 터미널 프로세스에 환경 변수를 나중에 주입할 수는 없습니다. task 제목을 바꿨다면 터미널을 다시 열거나 `CODEX_APP_THREAD_ID`를 사용해 주세요.

병렬로 실행하거나 한 번 쓰고 버리는 조사는 이 플러그인이 담당하지 않습니다. 이런 조사에는 `codex exec`를 직접 사용합니다.

## 개발

MCP server는 `src/`의 Rust crate로 빌드하며, `server` 실행 모드만 제공합니다. stdio MCP에는 공식 `rmcp` SDK를 사용하고, wire type은 Serde로 정의합니다.

- `src/client/mod.rs`: Desktop IPC facade
- `src/client/transport.rs`: endpoint와 framing
- `src/client/follower/`: task-state stream
- `src/client/lookup/`: 제목 조회에 쓰는 App Server의 수명
- `src/turns/`: facade, persistent task, registry로 turn의 수명을 관리합니다.

```sh
cargo fmt --check --manifest-path plugins/codex-app/Cargo.toml
cargo test --manifest-path plugins/codex-app/Cargo.toml
cargo clippy --manifest-path plugins/codex-app/Cargo.toml --all-targets -- -D warnings
cargo build --release --manifest-path plugins/codex-app/Cargo.toml
claude plugin validate .
```

릴리스할 때는 Apple Silicon Mac에서 release binary를 빌드한 후 `plugins/codex-app/bin/codex-app`으로 복사하고, 실행 권한을 `755`로 맞춰 주세요. 소스, release binary, `plugins/codex-app/.claude-plugin/plugin.json`, marketplace의 버전을 함께 올려야 합니다.
