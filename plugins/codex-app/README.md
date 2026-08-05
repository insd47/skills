# codex-app

Codex App 작업을 Claude Code background task로 위임하고 이어서 조율하는 Claude 전용 플러그인이다. persistent 스레드의 맥락 위에 쌓여야 하는 구현은 `ask`로, 병렬 일회용 조사는 `run`으로 라우팅한다. 협업 규칙 전체는 [`insd47` 플러그인의 `codex-app` 스킬](../insd47/skills/codex-app/SKILL.md)에 있다.

## 설치

marketplace를 등록하고 플러그인을 설치한 뒤 `/reload-plugins`로 현재 세션에 반영한다.

```sh
claude plugin marketplace add insd47/skills
claude plugin install codex-app@insd47-skills
```

업데이트는 marketplace snapshot을 갱신한 뒤 플러그인을 최신 버전으로 올리는 방식으로 적용하며, 세션을 다시 시작하거나 `/reload-plugins`를 실행해야 반영된다.

```sh
claude plugin marketplace update insd47-skills
claude plugin update codex-app@insd47-skills
```

## 명령

```text
/codex-app:ask <prompt>
/codex-app:run [--model <model>] [--effort <effort>] [--sandbox read-only|workspace-write|danger-full-access] <prompt>
/codex-app:status [job-id]
/codex-app:steer <job-id> <prompt>
/codex-app:interrupt [job-id]
```

## 동작

`ask`와 `run`은 Node 프로세스를 직접 detach하지 않는다. Claude Code가 Bash 호출을 background task로 소유하고, 스크립트는 Codex turn이 완료될 때까지 foreground에서 실행된다. 완료 시 셸 프로세스가 최종 결과를 출력하고 종료하므로 Claude가 별도 polling 없이 작업 루프로 복귀한다.

`ask`를 다시 호출하면 같은 persistent 작업의 실행 중인 turn을 자동으로 steer한다. 병렬 `run` 작업은 job ID와 함께 `steer`를 사용한다. `interrupt`는 Codex turn만 정상 중단하고 task는 보존한다. 강제 프로세스 종료와 별도 result 조회는 제공하지 않는다. background task 완료가 결과 전달과 루프 재개를 모두 담당한다.

`ask`는 현재 Codex Desktop이 소유한 task에 `${CODEX_HOME:-$HOME/.codex}/ipc/ipc.sock`을 통해 turn을 전달하므로 프롬프트와 진행 상황이 Desktop에 바로 표시된다. Codex Desktop은 내장 터미널을 만들 때 대화 제목을 `CODEX_APP_TITLE`로 전달하며, 플러그인은 이 제목과 정확히 일치하는 비아카이브 task가 하나일 때만 자동 선택한다. 제목이 없거나 일치 결과가 모호하면 조회 조건을 추측하지 않고 task ID를 요구한다. 명시적으로 고정하려면 Claude를 시작할 때 `CODEX_APP_THREAD_ID=<id>`를 설정한다.

Codex나 Claude는 이미 실행 중인 사람용 내장 터미널 프로세스에 환경 변수를 사후 주입할 수 없다. `CODEX_APP_TITLE`은 터미널 생성 시점의 스냅샷이므로 task 제목을 바꾼 뒤에는 터미널을 다시 열거나 `CODEX_APP_THREAD_ID`를 사용한다.

Desktop IPC는 현재 앱이 창간 task coordination에 사용하는 비공개 프로토콜이다. 플러그인은 앱과 동일한 길이 프레이밍 및 요청 버전을 사용하고, socket 소유권과 디렉터리 권한을 검사한다. 앱 업데이트로 프로토콜이 달라지면 별도 App Server task로 조용히 우회하지 않고 오류를 반환한다. `CODEX_APP_IPC_PATH`로 endpoint를 명시할 수 있지만 일반 설치에서는 canonical 경로를 그대로 사용한다.

`run`은 Desktop 실시간 연동을 사용하지 않는다. `CODEX_APP_BIN`, `CODEX_CLI_PATH`, `PATH` 순서로 Codex CLI를 찾아 독립 App Server task를 만들고, 결과를 전달한 뒤 자동 아카이브한다. 앱 번들 내부의 절대 경로에는 의존하지 않는다.
