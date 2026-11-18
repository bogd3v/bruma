# Micelio engineering standard: Rust profile

Profile `rust` of standard version 2.1 (2026-11-11). Canonical copy: `bogd3v/micelio-engineering`, `docs/standard/rust.md`.

How a repository in Rust meets the core, `docs/engineering-standard.md`. A repository that pins it is bound by exactly what it pins. Each section has the number of the section of the core it belongs to, and "section N" means that section of the core. The sections are those of `typescript.md`; a rule of that profile that has no counterpart here says why in its place.

## 4. Code

### Rust

- Every crate **must** build with `cargo clippy --all-targets --locked -- -D warnings` (a warning is an error) and be formatted by `rustfmt`. The check command covers every crate of the repository, tests included.
- No `unsafe` without a `// SAFETY:` comment that says why the code is sound; `unsafe` that a safe API can replace is replaced. A crate with no need for `unsafe` says `#![forbid(unsafe_code)]`.
- Data that crosses a boundary (HTTP, CMS, storage, environment) enters as bytes or text and is parsed into a typed value by a schema or a parser that returns a `Result`; nothing past the boundary handles raw input.
- Public functions and types declare their types, as every Rust item does. Type-only imports do not exist in Rust, so that line of the TypeScript profile does not apply.

### Where types and constants live

Reach decides the place, and visibility is the reach. A declaration starts private in the file that uses it and is promoted when its reach grows.

| Reach | Lives in |
| --- | --- |
| One file | That file, private |
| One function's parameters or result, used only with that function | Next to the function, `pub` with it |
| Several files of one module | That module's `types.rs` or `constants.rs`, `pub(super)` or `pub(crate)` |
| Several modules, or data that crosses a boundary (HTTP body, storage, configuration) | A module of its domain at the crate's top level, such as `post.rs` or `site.rs` |

- Modules are organised by domain (`post`, `site`, `theme`), never by kind of declaration and never one declaration per file. No `common`, `misc` or `shared` modules.
- When a domain module passes about 200 lines it becomes a folder with a `mod.rs` or a file of the same name that re-exports it. Consumers import from the module, so a move never touches them.
- A types module holds declarations only: no logic and no side effects.
- A shared constant is a value with a meaning beyond one function: a limit, a path, a key, a prefix. A literal that only makes sense inside one function stays with it, named, at the top of its file.
- A self-contained unit keeps its own copies on purpose: a migration, a plugin, a theme package.
- Export only what another crate or module uses: the default is private, then `pub(crate)`, and `pub` is the crate's API. An item nothing uses is removed (`dead_code` is a warning, so clippy with `-D warnings` fails it for a private or `pub(crate)` item; for a `pub` item of a library review does).
- Each `AGENTS.md` names the shared modules of its repository. A check of the import graph that fails when a type or constant declared outside these places is used by two or more modules does not exist for Rust as a standard tool: the visibility rules above and review hold it, and a repository that writes the check adds it to its check command.

### Style

- Each repository has exactly one formatter configuration, `rustfmt.toml` or none, checked in CI by `cargo fmt --check`. Formatting is never discussed in review: if `rustfmt` accepts it, it is fine.
- `async fn` with `.await` and `?`. A future is never driven by hand with `poll` or `block_on` outside a runtime entry point or a test. The `.then()` chains of the TypeScript profile do not exist in Rust, so that line does not apply.
- Pure logic lives in small functions with no framework types and is unit-tested there. Handlers, commands and routes stay thin.
- A file **should** stay under about 500 lines and do one job; split it when it passes either limit.
- Within a file the order is: `use` lines, types, constants, the public API, private helpers, then `#[cfg(test)] mod tests`.
- Dead code is deleted, not commented out. Git keeps the history.

### Errors and logging

- Errors are typed results: a function that can fail returns `Result<T, E>` with an error type that says what failed, and `?` carries it up. A library defines its error enum; an application may use one boxed error at its edge.
- No panic on input. `unwrap`, `expect`, `panic!`, `unreachable!`, indexing and arithmetic that can overflow are not used on a value that comes from outside the process; they are fine on an invariant the code itself guarantees, with a comment that says which. A test may use them.
- Catch an error only where it can be handled or given context. Never swallow one: an ignored result (`let _ =`) needs a comment that says why nothing is lost.
- At a boundary, turn the error into the right answer for the caller (an HTTP status and code, an exit code and a message, a neutral fallback) and log the cause.
- "Not found" is a normal result, an `Option` or a variant, not an error.
- Use the repository's one logging facade (`log` or `tracing`). `println!` and `dbg!` do not ship outside the output of a command-line program.
- Logs **must not** contain secrets, tokens, email addresses or raw IP addresses.

### HTTP APIs

- An API that Micelio designs answers an error as a problem detail (RFC 9457, `application/problem+json`): `type`, `title`, `status`, `detail`, and a stable, machine-readable `code` as an extension member. A client branches on `code`, never on the text.
- Its reference is an OpenAPI document generated from the types that validate its requests and responses. A check fails when the document drifts from them.
- An API whose format belongs to a framework keeps that format.

### Doc comments (rustdoc)

Doc comments follow [rustdoc](https://doc.rust-lang.org/rustdoc/). They are what an editor shows on hover, what an agent reads before calling a function, and what the API reference is generated from.

What **must** have one:

- Every public item: function, method, type, trait, constant, module and macro, and each public field or variant whose meaning its name and type do not make obvious.
- The crate itself, with `//!` at the top of `lib.rs` or `main.rs`.

The component props and emits of the TypeScript profile do not exist in Rust: the fields and variants above are their counterpart. Its `@public` and `@internal` tags and the default export a framework discovers have none either: `pub` against `pub(crate)` says what is public, a handler a framework discovers is documented in `docs/api.md` as there, and a theme contract, which is a TypeScript matter, has no Rust slot API.

A private item gets one only when it helps. `#![deny(missing_docs)]` in a library crate, and `cargo doc` with warnings denied, check it (section 15).

How it is written:

- The first sentence is the summary: what the thing is, returns or guarantees, in the present tense, able to stand alone in a list. When that is all there is to say, the comment is one line: `/// …`.
- After it comes only what the signature cannot say: when the result is `None`, side effects, units, caching, who may call it, why it exists.
- Never repeat the name or the types. Identifiers go in backticks and intra-doc links, `` [`Name`] ``. Point to an ADR or a document instead of copying it.
- A section is used only when it adds information:

| Section | Use it for |
| --- | --- |
| `# Errors` | Every error the caller is expected to handle, with its variant or code, in a function that returns `Result` |
| `# Panics` | Every input that makes the function panic; a public function that can panic on outside input breaks the rule of "Errors and logging" |
| `# Safety` | What the caller must guarantee, in an `unsafe fn` (the `// SAFETY:` comment of "Rust" covers a block) |
| `# Examples` | An input format or a call that is not obvious; the code is fenced and runs as a doctest |

- `#[deprecated(since = "…", note = "…")]` names the replacement and the release that removes it (section 7).
- A comment changes in the same pull request as the behaviour it describes. A comment that is wrong is a bug.
- Lint checks that public items have a comment (`missing_docs`), `cargo doc` that the links resolve (`rustdoc::broken_intra_doc_links`), and `cargo test` that the examples run.

### Comments and suppressions

- Comments inside a function are short and say why, not what. Anything longer goes in `docs/` or an ADR and the comment points to it.
- A `TODO` names its issue: `// TODO(#123): …`. Without an issue it is not a plan.
- A suppression (`#[expect(lint, reason = "…")]`, or `#[allow(lint)]` where `expect` cannot be used) covers one item and carries its reason. `#![allow(warnings)]`, `#![allow(clippy::all)]` and crate-wide allows are not used.

## 8. Tests

- Levels, from cheapest to most expensive: unit (`#[cfg(test)]` next to the code), doctests, integration (`tests/`, through the public API), end to end and performance budgets. `AGENTS.md` lists the commands of each repository.
- Prefer the real thing to a mock. Replace a collaborator only at the process boundary (the network, the clock, the file system), with one shared fake.

## 10. Dependencies, licences, supply chain and runtime

- A new dependency **must** be justified in the pull request: what it replaces, what it weighs, its licence and who maintains it. Prefer the standard library (principle 5).
- Licences are checked against an allowlist that the repository **must** have, keep in a file (`deny.toml`) and check in CI (`cargo deny check licenses`). The profile requires the allowlist and that it is checked, not its contents: the policy is the repository's, or Micelio's in the section "Micelio only". Third-party material keeps its licence file next to it, is listed in `THIRD-PARTY.md` and annotated in `REUSE.toml`.
- `Cargo.lock` is committed, for a library as for a program, and never edited by hand. CI builds and tests with `--locked`.
- The toolchain is pinned in `rust-toolchain.toml` by an exact version and lists `clippy` and `rustfmt` as components. CI installs what that file names; the lowest supported version, when it differs, is `rust-version` in `Cargo.toml` and is tested.
- Pin exactly what breaks on minor versions (a `0.x` crate, a crate that links a native library); use caret ranges for the rest.
- Dependabot proposes grouped minor and patch updates weekly (`cargo` ecosystem). A major upgrade is done by hand, in its own pull request, with the full suite.
- Known vulnerabilities are checked on every pull request and on a schedule with `cargo audit --deny warnings`: an advisory, an unmaintained crate or a yanked one fails it. An advisory that cannot be fixed yet is written down with the reason the risk is accepted.
- GitHub Actions are pinned by commit SHA, with the version in a comment.
- Every published artifact (a crate, a binary, an image) carries a software bill of materials and a signed build provenance attestation, and the operator documentation says how to verify them.
- The OpenSSF Scorecard runs on every repository. A finding is fixed or written down with the reason, like a dependency alert.
- All repositories of one organisation run the same Rust toolchain unless an ADR says why not. `rust-toolchain.toml` is the source: CI reads it, and `rust-version` matches or lowers it.

## 11. Git, pull requests and review

### Pull requests

- The check command of `AGENTS.md` passes locally before the pull request is opened. CI is the gate; a red check is never merged around.

### Done

Besides the items of section 11, "Done":

- New and changed public items have their rustdoc comment.

## 12. Documentation, decisions and planning

### Organised by reader

- Reference is generated from its source of truth whenever one exists, and a check fails when it drifts. For code, the source of truth is the rustdoc comment (section 4).

## 14. Starting a new repository

The template of `micelio-engineering` (`template/`) produces the Node form of this list; for Rust the list is what the repository must contain.

- [ ] `LICENSE`, `REUSE.toml`, `THIRD-PARTY.md` and the licence allowlist (`deny.toml`) with its check
- [ ] `README.md` and the short `CONTRIBUTING.md` that links Micelio's canonical one (section 1); `SECURITY.md` and `CODE_OF_CONDUCT.md` only when the repository has something to add to the organisation's
- [ ] `AGENTS.md` with the pinned version of this standard and `profile: rust`, `CLAUDE.md`, `.claude/TEAM.md` and the copies of the core and of this profile
- [ ] `.editorconfig`, `rust-toolchain.toml` and `.env.example`
- [ ] A `version` in `Cargo.toml`, `rust-version`, the committed `Cargo.lock`, and the release workflow of section 7
- [ ] In `AGENTS.md`, the `check:` line of the first `## Commands` block, as section 15 says, and the commands to build, run and test
- [ ] CI with the required checks of section 15 called from the reusable workflows of `micelio-engineering` with `profile: rust`, the DCO check, Dependabot and the Scorecard workflow
- [ ] The pull request template, the issue templates, the title-to-label workflow and the release notes configuration
- [ ] `main` protected by the organisation rulesets: changes arrive by pull request, required checks pass, no force push
- [ ] The Micelio bot installed on the repository (section 16)

The block "A plugin repository also needs" of `typescript.md` is for Strapi plugins and does not apply: there is no Rust plugin scaffold in the standard.

## 15. Required checks

Every repository has one check command, the `check:` line of the first `## Commands` block of its `AGENTS.md`, and it runs the fast checks in the order CI does. The reusable workflow runs that line (`docs/workflows.md`, "`rust`").

| Check | Command or workflow | In `check` |
| --- | --- | --- |
| Format | `cargo fmt --check` | Yes |
| Lint, including the repository's own rules | `cargo clippy --all-targets --locked -- -D warnings` | Yes |
| Tests, doctests included, and the coverage floor when the repository has one | `cargo test --locked`; `cargo llvm-cov --locked --fail-under-lines <n>` for the floor | Yes |
| Public documentation | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked` | Yes |
| Licences | `cargo deny check licenses` and `reuse lint` | Yes (allowlist) |
| The copy of this standard | `npx --yes --package @micelio/eslint-config@<version> micelio-lint-standard`, or the `lint-standard` action in CI | Yes |
| Build | `cargo build --locked --release` | No |
| Known vulnerabilities | `cargo audit --deny warnings` | No |
| Sign-off | The `DCO` workflow | No |
| The repository's own suites | Integration, end to end, performance budgets | No |
| The contract with the other repository | The contract suite against a real CMS: Micelio's contracts (section 6) are between TypeScript repositories, so a Rust repository has none unless an ADR adds it | No |
| Supply chain | SBOM and provenance on every published artifact; the Scorecard workflow | No |

## 16. Changing this standard

- `lint:standard` fails when the local copy of the core or of this profile does not match the version pinned in `AGENTS.md`. A repository may stay on an older version on purpose; the lint checks the pinned one. A repository without npm runs it as the row "The copy of this standard" of section 15 says, and the sync (`docs/standard-sync.md`) moves the pin and the files without touching `Cargo.toml` or `Cargo.lock`.
