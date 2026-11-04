---
id: 082
title: Reid visual foundations and native shell
status: in-progress
owner: codex
stage: 9
depends_on: [081]
---

## Scope

Port the adopted theme to production tokens and shared primitives; restyle the current shell and existing screens. Keep existing domain/IPC contracts until their replacements are locked. This task does not fabricate numeric mastery or add an adaptive engine.

Product authority: reference/UI/AXIOM-HANDOFF.md and
docs/superpowers/specs/2026-10-10-reid-design-adoption.md.

## Plan

- src/styles/tokens.css and global.css; existing primitive/badge/mastery CSS Modules.
- AppShell / Sidebar / shared inspectors and relevant layout styles.
- Render the five adopted references and compare live app surfaces.
- Validate macOS native controls and Windows/Linux chrome; add tokens for opaque glass fallback.

Acceptance:
- Centralized tokens, readable focus/contrast, responsive panels and working controls.
- Existing first-launch/create/resume/check/hint flows still work.
- Conditional variants have render tests; frontend gates pass and native smoke flow remains green.

### Implementation plan (2026-11-03)

Human reassigned the visual lane to Codex. Brainstorm: preserve the existing operations and native decorated window, apply the adopted theme through the existing semantic token vocabulary, and avoid any new learning-evidence contracts. The accepted handoff and 2026-10-10 adoption spec supply the specification; no new architectural decisions are required.

1. Port Reid surfaces, text, actions, typography, metrics, radii and shadows into src/styles/tokens.css; retain existing token names for all consumers. Add opaque, reduced-motion, forced-color and focus fallbacks in tokens/global styles.
2. Restyle primitives, badges, qualitative mastery, AppShell/Sidebar, layouts and overlays using these tokens. Preserve platform-native decorated window controls and the existing drag strip; add no simulated traffic lights.
3. Reskin existing pages through shared tokens plus targeted card, heading, form and feedback styles. Keep services/types/hooks/persistence untouched. No new Today/Profile/History routes or invented numeric evidence.
4. Run frontend, Rust and native E2E gates; record unavailable platform checks explicitly. Push the task branch and open a PR for independent review. Native platform/fidelity acceptance remains pending wherever this environment cannot exercise it.

## Worklog

- 2026-10-10: Scoped following the human's full design/product adoption in task 081.
- 2026-11-03: Human reassigned 082 to Codex. Read AGENTS, CLAUDE, ARCHITECTURE, OWNERSHIP, Stage 9, all .ai process documents, handoff and adoption spec. Claimed in-progress on agent/codex/082-reid-visual-foundations. Task 083 contract docs excluded.
- 2026-11-03: Ported the Reid theme into the existing production token vocabulary. Added separate readable success/progress text tokens, focus, motion, opaque/glass surfaces, forced-color overrides and missing legacy spacing/title tokens. No reference CSS imported into production.
- 2026-11-03: Reskinned primitives, badges and qualitative mastery; replaced arbitrary quarter-filled mastery glyphs with categorical ring treatments. Updated floating sidebar, shell gutters, inspectors, sheets/palette, raised cards, editorial headings, answer entry and truthful check-result feedback. Existing destinations and all data/service/type contracts preserved. Native OS decorations and the drag strip retained; no fake traffic lights.
- 2026-11-03: Rendered Main, Session, Feedback, Profile and Summary references at 1280px with their local CSS inlined; all rendered without console messages. Rendered the production first-launch bundle with an isolated fixed 800px preview height (production viewport sizing unchanged), confirming dark zero-workspace presentation. This is browser rendering, not a native IPC or platform smoke test. Populated existing screens and all-platform fidelity still need manual acceptance.
- 2026-11-03: Frontend gates passed; full suite 62 files / 209 tests, plus targeted StudySessionPage 7 tests after status assertions. Added 44 real token contrast checks at WCAG AA small-text threshold. First contrast-test attempts exposed the test runner's empty CSS imports / transformed import.meta URL; switched to the repo's existing filesystem-test approach, then all passed. One concurrent lint attempt raced Vite's temporary config deletion; sequential lint rerun passed without product changes. CI runs these gates sequentially, so no persistent application flake was identified.
- 2026-11-03: Rust and native gates attempted but blocked by local prerequisites, detailed below. Kept status in-progress because quality-gates.md prohibits review until applicable gates pass. Delivery is a draft PR pending native/CI and independent visual review; no self-approval or merge.

## What was built / tested / left out

Built:
- One Reid production theme inherited by all existing screens; targeted raised-card treatments on Home, overview, concepts, material, tools, first launch, session, full visualization and goal editing. Marketplace, module and concept detail retain their layouts and inherit the shared palette, typography, primitives and inspector treatment.
- Rounded controls and badges, high-contrast actions and textual feedback; categorical legacy mastery, without invented scores. Responsive Home cards and stacked two-pane inspectors. Opaque fallback before backdrop-filter support; reduced-transparency, reduced-motion, forced-color and visible keyboard-focus treatments.
- Existing first-launch/create/resume/check/hint behavior retained. No new domain data, hooks, service signatures, shared types, persistence or architecture rules. No structural doc changes; task 083 docs untouched.

Validation (Windows host, 2026-11-03):
- PASS: npm run typecheck; npm run lint (sequential rerun); npm run build.
- PASS: npm test, 62 files / 209 tests. Existing fixture-backed page, hook, component and app tests cover creation/navigation, resume, checks and hints. Targeted StudySessionPage suite: 7 passed. New contrast suite: 44 passed.
- PASS: design-value scan (hex / rgba outside tokens.css) found no matches; all CSS token references resolve; no production component imports services; git diff --check clean.
- BLOCKED: cargo check --locked and cargo test --locked, via --manifest-path src-tauri/Cargo.toml: link.exe missing. Visual Studio Build Tools/vswhere also absent. No Rust code changed.
- BLOCKED: cargo clippy --all-targets --locked -- -D warnings: clippy component absent. cargo fmt --all --check: rustfmt component absent.
- BLOCKED: npm run test:e2e:linux: xvfb-run is unavailable on this Windows host; native E2E did not execute. Native release smoke, restart persistence, network-disabled acceptance (068) and Rust regression corpus (067) are not claimed as passing.
- VISUAL: five references and the production first-launch bundle rendered; macOS/Windows/Linux native window control operation, reduced-transparency/forced-color interaction and populated-screen fidelity are not verified here.

Left out deliberately: new Today/product-area navigation (084), evidence contracts (083), numeric mastery/diagnostics/difficulty/adaptive queues (085), profile/history/summary features (086). Existing empty/unavailable surfaces remain; no sample reference observations were imported. Implementation stays in one PR; outstanding work is acceptance evidence, not a newly invented contract.

PR: pending creation.

## Review

Pending implementation and independent review.

## Follow-ups

- Run required CI/native gates in an environment with the documented toolchain, then move to review only when green.
- Complete native controls and populated-screen visual acceptance on supported platforms, including accessibility preferences.
- Independent reviewer must apply .ai/review-checklist.md; .ai task changes require human sign-off before merge.
