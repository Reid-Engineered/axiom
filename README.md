# Axiom

A desktop learning environment built around one idea: a learner has a **goal**, works
inside a **workspace**, and everything the app shows — recommendations, tutoring, practice —
is in service of that goal, expressed in the learner's own terms. No dashboards, no
percentages, no streaks or badges (Unless the user sets it up). Progress is a named mastery state and a sentence, never a
score.

Built with [Tauri](https://tauri.app/) (Rust backend, React/TypeScript frontend), local-first
and offline-first — SQLite is the source of truth on disk, and there is no server.

## Status

Pre-release, actively in development. The frontend is complete through Stage 6 of the
[roadmap](ROADMAP.md); Stage 7 (SQLite persistence, IPC commands) is in progress. See
[ROADMAP.md](ROADMAP.md) for stage-by-stage scope and acceptance criteria.

## Getting started

Requires [Node.js](https://nodejs.org/) 18+ and the
[Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform (a Rust
toolchain plus your OS's native webview dependencies).

```bash
npm install
npm run dev        # frontend only, in a browser
npm run tauri dev  # full app in a native window
```

Other useful scripts:

```bash
npm run typecheck   # tsc --noEmit
npm run lint        # eslint
npm run build       # tsc + vite build
npm run test        # vitest
npm run test:e2e:linux   # native end-to-end flow (see e2e/README.md for driver setup)
```

## Documentation

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — system structure, data flow, folder layout.
- [`AGENTS.md`](AGENTS.md) — engineering conventions and design rules for anyone (human or
  AI) contributing code.
- [`ROADMAP.md`](ROADMAP.md) — staged delivery plan and acceptance criteria.
- [`reference/UI/AXIOM-HANDOFF.md`](reference/UI/AXIOM-HANDOFF.md) — the product and visual
  design spec.

## License

[PolyForm Noncommercial 1.0.0](LICENSE) — you're free to read, run, and modify this code for
any noncommercial purpose. Commercial use is not permitted without a separate agreement.
