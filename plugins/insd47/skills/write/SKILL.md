---
name: write
description: Writes prose in Insung Hwang's repositories - README and `.docs` markdown, doc comments and plain code comments (including whether a comment belongs at all), error messages and user-facing strings, commit messages, and pull request bodies - following the repository's own conventions first and a shared Korean style guide. Use when creating or editing any markdown document, adding or revising a comment, wording an error message or string, committing, or writing a PR, including while implementing code.
---

# Write

## Whose conventions

Explicit requests from the user come first, then the repository's AGENTS.md or CLAUDE.md. Below that, who maintains the repository decides:

- **The maintainer's own repository**, where they are the primary author in the git history, or a new repository with no conventions yet: this skill is the convention. Text you write takes the shape described here even where older material differs, because the older material may predate these rules or come from unreviewed generated changes. That never widens the change to untouched material.
- **A repository someone else maintains**: its conventions win. Match the closest material first, then the module, then the project. Use this skill only for decisions the codebase leaves open, and never restructure existing work toward it.

A PR template counts as the repository's convention. Korean text follows [KOREAN.md](KOREAN.md); read it completely before writing Korean. Its clause that comments and commit messages keep the project's existing conventions agrees with the order above. The samples below come from documents the maintainer corrected by hand, so match their shape rather than paraphrasing the rules.

On top of KOREAN.md, the maintainer's register:

- Sentences in 합니다체, doc and code comments included. Requests to the reader end in `~해 주십시오` or `~해 주세요`; cautions stand apart as `>` quotes.
- Field, argument, and return descriptions are short noun phrases without a period, optionally followed by one full sentence. Commit subjects end in a noun form. Both are deliberate exceptions to KOREAN.md's rule that sentences end in a predicate.
- Particles attach to English terms and code tokens without a space: `Sonner로`, `` `Status`로 ``.
- Avoid em dashes and arrow chains between clauses, heavy bold, dropped particles and stacked nouns, home-made metaphors (`손잡이`, `흘려보낸다`), and detail a reader could get from the code.

## Comments

Whether a comment exists depends on who reads the code.

- **Published libraries** document their public surface. A type or function gets one summary sentence, a blank line, then `- ` bullets for contracts a caller must know. Fields and arguments are noun phrases, optionally followed by one sentence. Module charters (`//!`) state what the code cannot show: an invariant, a deliberate absence, a boundary contract.

  ```rust
  /// Runner 호출 하나를 취소하기 위한 Presigned URL 쌍입니다.
  ///
  /// - 두 URL은 `Signals` 버킷의 같은 무작위 key를 가리킵니다.
  pub struct Cancel {
      /// 채점 결과가 확정되면 SDK가 빈 본문으로 PUT할 URL. Runner에는 전달하지 않습니다.
      pub signal_url: String,
  }
  ```

  ```ts
  /**
   * 배포할 때 값을 자동으로 만드는 Secret을 생성합니다.
   *
   * - 값은 영숫자 64자이며, 리소스를 교체하지 않는 한 바뀌지 않습니다.
   */
  export class RandomSecret extends $util.ComponentResource {
    /** 비밀값 */
    readonly value: $util.Output<string>;
  }
  ```

- **Deployed applications** carry almost no doc comments. A plain `//` comment appears only where the code cannot show its reason: a platform constraint, an external fact, a deliberate choice a future reader might undo. It is one plain sentence that states the reason.

  ```rust
  // 연결을 재사용하여 S3 주소 조회를 줄입니다. VPC DNS는 ENI당 초당 패킷 수가 제한되어 있습니다.
  // 확인에 실패하면 취소되지 않은 것으로 처리합니다. 취소는 리소스 절약을 위한 것이며, 안전장치가 아닙니다.
  ```

- Record a deliberate, non-obvious choice where a future reader would otherwise undo it, including two similar paths that coexist on purpose; put it exactly where that reader will look. Routine decisions need no catalog of rejected alternatives.
- Existing comments belong to the maintainer, who often edits them by hand. During structural work, move them with their code instead of deleting them, and report the locations of any that became stale.
- Never restate the next line. No TODOs, banners, or commented-out code. A route handler carries at most one behavior sentence, never the method or path the router already declares. In JSDoc, document a destructured option as `@param property`, not `@param param0.property`.

## Documents

A README tells a user what the project is and how to use it: a one-sentence identity, structure, installation, a usage sample whose comments explain each step, and constraints. Maintainer knowledge (measured limits, the reason behind each design step, rejected designs) goes to `.docs/`. State measurements with their conditions, and give the reason for each rule the reader must not break.

```markdown
응답 스트림을 끊어서는 Runner를 멈출 수 없으므로, 취소 신호를 별도로 전달합니다.

> 호출자는 두 Lambda의 `lambda:InvokeFunction` 권한이 필요합니다.
```

## Error messages

What an error carries is decided while building (see the build skill's error rules). The wording is one sentence in 합니다체 without a trailing period, because it is prefixed onto a cause chain: `context("입력의 압축을 풀지 못했습니다")`. A message shown to an end user explains the situation and, when there is one, what they can do.

## Commits

In the maintainer's own or a new repository:

- `type: 제목`: a conventional type and a short Korean noun-form subject, no scope, no body. `feat: 대회 안내에 설문조사 URL 설정`, `fix: 코드블럭 복사에 줄바꿈이 반영되지 않는 문제`.
- One concern per commit; split unrelated changes instead of listing them in one subject.
- The first commit of a repository is exactly `initial commit`.
- A pull request lands as a merge commit titled with a noun-form phrase and its number, without a type: `채점 기능 추가 #140`.
- Planning notes such as `.plan/` stay out of commits.
- Trailers the harness is configured to add stay as configured.

A pull request body, absent a template, is a few sentences on what changed and why, followed by how it was checked.
