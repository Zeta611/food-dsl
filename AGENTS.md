# Agent instructions

Food DSL is a language for designing 3D-printed food. It has a Rust core compiled to WASM and an interface built with Elm and three.js. Setup steps are in `README.md`.

For contributors who are designers rather than programmers, we explain changes and trade-offs in plain language.

## Where to work

| Path | What it is | Rule |
| --- | --- | --- |
| `ui/` | The interface. Elm talks to three.js through the `render-preview` custom element. | Open a PR. |
| `dsl/`, `binding/` | The language core and its WASM API | Agree on the change in an issue before writing code. |

## Language design

The team's design draft defines the language, and the code implements only part of it so far.

- Don't treat a feature that's in the draft but not in the code as a bug, and don't implement it without agreement.
- Don't add or repurpose language constructs, or imitate them in the UI, to make a task work. Stop, tell your human what the language can't express, and offer to draft an issue.
- If you can't access the draft, ask your human rather than guessing what it says.

## Public API

Other projects use `Cup` and the functions exported from `binding/src/lib.rs`. Call out any change to them in the issue and PR.

## Making changes

- Never push to `main`, because that deploys to production. Work on a branch and open a PR.
- Keep each PR to one concern.
- Describe what changed and why in plain language, and include a screenshot of any visible change.
- Rebuild the WASM after changing `dsl/` or `binding/`.
- Run `pnpm check`, `pnpm typecheck` and `pnpm build` from `ui/`. They don't show that a shape renders correctly, and there are almost no automated tests, so open the app and look before calling a change done.

## This repository is public

Don't commit other people's photos, quotes or unpublished material, or any credentials.
