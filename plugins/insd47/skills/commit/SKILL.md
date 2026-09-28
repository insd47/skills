---
name: commit
description: Writes git commit messages and squash-merge titles in Insung Hwang's convention - a conventional type prefix with a short Korean noun-form subject and no body. Use when creating a commit, amending a commit message, or writing a merge commit in the maintainer's repositories.
---

# Commit

The repository's own documented convention takes precedence. Otherwise, write every commit the way the maintainer writes one by hand.

## Subject

`type: 제목` — a conventional type, a colon, and a short Korean noun-form phrase.

- Types: `feat` for new behavior, `fix` for broken behavior, `refactor` for structure without behavior change, `chore` for tooling, dependencies, and version bumps, `docs`, `style`, `ci`, `build`, `test`. No scope in parentheses.
- The subject ends in a noun form — `추가`, `수정`, `변경`, `개선`, `정리`, `제거`, `설정`, `전환`, `분리`, `해결`, `통합`, `반영`, `대응`, `표시` — or, for a fix, names the problem: `~하지 않는 문제`. Declarative endings (`~한다`, `~하게 한다`) are not used.
- Keep it to roughly 15–30 characters; technical nouns stay in their original spelling (`Sonner`, `CSP`, `Codex App IPC v2`), and Korean particles attach to them without a space: `Sonner로`, `CSP에`, not `Sonner 로`.

```
feat: 대회 안내에 설문조사 URL 설정
feat: 감독관에게 새 채팅을 Sonner로 알림
fix: 마지막 득점 시각을 감독관 시간대로 표시
fix: 코드블럭 복사에 줄바꿈이 반영되지 않는 문제
chore: frameless-window 업그레이드
```

Not this:

```
fix: 채점표 CSV 에서 제출이 없는 문제는 0점으로 채운다

제출이 없는 문제의 칸이 비어 있어 합계가 어긋났다. ...
```

## No body

The subject is the whole message. The reasoning lives in the code, the pull request, or the conversation, not in a commit body. Trailers the harness is configured to add are not part of this convention and are left as configured.

## Granularity and special cases

- One concern per commit; split unrelated changes rather than listing them in one subject.
- The first commit of a repository is exactly `initial commit`.
- A pull request lands as a merge commit titled with a noun-form phrase and its number, without a type: `채점 기능 추가 #140`, `대회별 감독관 콘솔 주소 분리 #131`.
- Planning notes such as `.plan/` stay out of commits.
