# Axiom — UI Design Handoff

**Adopted direction: Reid Engineered, 2026-10-10.** The human approved the full visual
design and product direction from PR #25, including numeric mastery, error profiles,
difficulty modes, and the new sidebar. This replaces the previous light, two-accent,
workspace-tree design restrictions.

This document is the implementation authority. The five screens and theme under
[2026-10-macos-reid](proposals/2026-10-macos-reid/README.md) are the visual references.
Their sample content illustrates behavior; it is not learner data or a working capability.
The prior handoff is preserved in [the legacy archive](AXIOM-HANDOFF-LEGACY-2026-10.md)
for historical context. Older screenshots cannot override this document.

Adoption changes the product target; it does not claim that the running application has
already migrated. Architecture §8 and Roadmap Stage 9 distinguish current contracts from
the new work.

## 1. Product model

- A **workspace** remains the persisted learning environment for a subject. The new UI
  calls these **Domains** in its sidebar; this is a presentation label, not a database
  rename. Domain selection visibly scopes Practice, Error profile, Concept map, and History.
- A **goal** retains the learner's natural-language intent, inferred deadline and scope.
  One Guiding goal shapes recommendations. Waiting, Met, and Resting goals remain available.
  The Next exam card reflects the active goal's deadline; it never invents a separate exam.
- A **concept / skill** has prerequisite relationships and learning evidence. Numeric
  mastery, change since previous practice, decay, readiness, and Bloom level are adopted
  presentation targets. Named mastery descriptions remain useful explanations.
- A **session** has an editable plan, named phases, reasons, question positions, difficulty,
  timing, and a persisted current attempt. Resume restores the same attempt and working.
- A **pattern** records a recurring misconception, its opportunities and occurrences,
  first/last seen, recent evidence, clean attempts, and Recurring / Improving / Resolved
  status. These are provider-produced conclusions, not deductions made by UI components.
- A **module** provides workspace-scoped capabilities. Practice and Error profile are
  stable product areas, not module identities. Installing a module does not add navigation.
- The app remains local-first and offline-first. A fresh install has no fabricated learner
  history, mastery, patterns, sessions, or sample workspaces. Sample exploration is explicit.

## 2. Design system

### Surfaces and signals

Use the adopted [theme](proposals/2026-10-macos-reid/axiom-theme.css) and
[component reference](proposals/2026-10-macos-reid/ds/reid/components/bundle.css).

| Role | Reference value |
|---|---|
| Background | #1e1b19 |
| Content surface / raised surface | #2a2724 / #302c29 |
| Sidebar / inspector | rgba(50,46,43,0.88) / rgba(44,41,38,0.92) |
| Primary / secondary / faint text | #f5f5f7 / rgba(235,235,245,0.70) / rgba(235,235,245,0.56) |
| Action accent / hover / text | #7ea6ff / #9bbaff / #8fb3ff |
| Danger / destructive fill | #ff6b5b / #d33a2c |
| Info / progress / success / neutral | #2f6fdb / #c2692f / #2f8a57 / #8e8e93 |

Dark surfaces, blue actions, glass sidebar/inspectors, rounded controls, and floating-panel
shadows are adopted throughout. Red marks misses or negative changes. Success, progress,
and neutral signals supplement the action accent. Color never carries a diagnosis alone.
Reduced-transparency environments use opaque equivalent surfaces with readable contrast.
Do not add unrelated gradients or glow beyond the supplied component treatment.

Port production values to src/styles/tokens.css. The reference CSS is a drawing resource,
not a second production token system. Components use semantic tokens and shared primitives.

### Typography and metrics

- UI: the theme's system stack, led by -apple-system / SF Pro Text with Windows and generic
  sans-serif fallbacks. Monospace is for answer entry and technical detail.
- Editorial headings and mathematics: the theme's New York / ui-serif / Iowan Old Style /
  Palatino Linotype / Georgia stack. Typeset actual expressions structurally; the hand-set
  fractions in the mockups are illustrations. Embed any required math assets for offline use.
- Sidebar 232px; inspector 300px; toolbar 64px; controls 34px (small 28px); rows 28px.
  Content gutters 28px; spacing scale 4 / 8 / 12 / 16 / 20 / 24 / 28 / 40px.
- Radii 3 / 6 / 8 / 12 / 16 / 24px and pill controls; use the theme's glass, float, sheet,
  and menu shadows. Typography follows the supplied screen/component references.

macOS chrome may use real native traffic-light controls and draggable regions. Windows and
Linux retain functioning platform-appropriate window controls. Do not draw inert traffic
lights as though they minimize, maximize, or close the application.

### Components and copy

Use shared buttons, segmented difficulty controls, search fields, sidebar rows, detail lists,
status labels, charts, stat tiles, and mastery meters. Before/after meters distinguish the
previous observation from the current value in text as well as color.

### Copy rules

Feedback may say Correct / Incorrect and must also name what happened and what to do next.
Numeric evidence, summaries, charts, and clean-attempt counts are permitted. Keep copy
concise and factual: no celebration, exclamation marks, emoji, or invented observations.
Explain checker conclusions accurately; never call numerical agreement symbolic proof.

## 3. Navigation

The primary sidebar is:

~~~
Today
Practice
Error profile
Concept map
History
── Domains ──
Calculus II · MTH 112
other learner-created domains
Add Domain…
── Next exam (when the Guiding goal has a deadline)
Settings
~~~

Search and its keyboard shortcut remain discoverable in the toolbar/command palette.
Practice, Error profile, Concept map, and History always show their domain scope. Domain
selection retains workspace identity, goal, and history. With zero domains, show Create
domain and Explore a sample workspace; scoped areas show an intentional empty state.

Settings provides domain tools, preferences, and access to Marketplace; material and notes
remain reachable from the domain/concept details and command palette. Existing destinations
must not become dead ends during migration. Contextual session phases, recent sessions, and
profile filters may occupy sidebar sections as the five references show. Modules never add
rows. Full Visualization remains a session detour with an explicit return.

## 4. Adopted screens

### Today — Main.dc.html

Dark app shell and floating sidebar; compact toolbar with remembered date/context, search,
and New session. Show an existing resumable session before offering a new plan. The plan
states duration, question count, difficulty, and Warm-up / Stretch / Combined phases with
reasons drawn from actual history. Start session and Change plan are real actions.

Watching for surfaces named patterns with evidence; Fading shows previous/current mastery
and elapsed practice time. A domain concept map shows prerequisite readiness and current
mastery. Next exam reflects the Guiding goal. Empty evidence produces an empty state, not
the mockup's populated Calculus II profile.

### Practice — Session.dc.html

Question-first working area with named phase, position, Bloom level, difficulty, actual time,
End session, and segmented question progress. Render prompt/domain restrictions, answer
entry, parsed preview when supported, Hint, Skip, and Check answer.

A dismissible Why this question inspector describes actual selection reasons, watched
patterns, prerequisites, prior practice, and the difficulty policy. Technical template and
checker detail can live behind an optional detail disclosure. Tutor and visualization are
contextual, optional tools; the former visualization-dominant layout is no longer mandatory.

### Feedback — Feedback.dc.html

Keep the question and learner answer visible beside the expected result or a safe
explanation. A red status accompanies a named misconception. Show the matched pattern,
mastery/Bloom changes, clean-attempt changes, and what the response changed in the remaining
plan. Worked solution, pattern detail, and Next question must have working destinations.

Apply changes once per submission. Never display a swapped question, score update, or
diagnostic claim until its persisted capability response confirms it. Partial success
(for example, a correct antiderivative missing its family constant) is explained accurately.

### Error profile — Profile.dc.html

A domain-scoped first-class page with Recurring / Improving / Resolved / All filters and
search. Rows name patterns, opportunities/occurrences, status, and dated evidence. Selection
opens a dismissible detail inspector with recent attempts, an occurrence chart, resolution
policy, Drill this pattern, and Edit rule. Resolved patterns retain their history.

Opportunity counts include only problems capable of testing the pattern; absent observations
are not clean attempts. A final expression alone cannot prove which substitution or method
the learner used. Unsupported attribution is unknown, not an inferred fact.

### History / session summary — Summary.dc.html

History lists real sessions. A completed session opens a summary with elapsed time,
difficulty, correct/total, hint usage, mastery changes, and repeated patterns. The question
table traces phase, prompt, result, and before/after mastery. Profile changes and the next
recommended session explain concrete consequences. Review misses and Done are actionable.

### Remaining screens

First Launch and Create Workspace keep their real zero-domain/create flow in the new visual
language. Workspace Overview, Concept View, Material, Tools, Marketplace, Module Detail,
Goal Editing, Command Palette, and Full Visualization retain their useful operations while
adopting the same theme and navigation. Concept View can show numeric mastery and evidence
beside explanations. Goal editing still previews consequences and preserves all history.
No screen ships sample evidence merely to match the drawing.

## 5. Adopted mechanics and implementation boundaries

| Mechanic | Adopted target | Contract work required |
|---|---|---|
| Mastery and decay | Numeric values, before/after changes, meters, readiness and Bloom progression | Versioned evidence policy; persistence; explainable updates and decay |
| Pattern resolution | Configurable opportunity-based clean-attempt rule and status; mockup uses five consecutive clean attempts | Opportunity tagging, event history, reset/recurrence rules, persistence and policy editing |
| Adaptive next question | A miss may replace the next question to retest its pattern; show the change | Persisted queue, reasons, resume semantics, skip/retry behavior and content availability |
| Difficulty | Steady / Push / Exam, explicit hint budgets and mastery costs | Configurable provider policy and disclosed stakes; test effects of hints and partial success |
| Attribution | Per-problem pattern opportunities and supported diagnostic evidence | Knowledge Package schema/content and Practice diagnostics; no subject logic in Core |
| Checker detail | Truthful method, domain, sample/term evidence when supported | Versioned verifier output; optional technical disclosure; no fabricated proofs |

The mockup's 0.05 costs, five-attempt rule, Bloom thresholds, timings, sample counts, and
example questions are candidate policy values, not validated calibration. Approving the
direction permits these mechanisms; implementation tasks must lock and test actual policies.
Do not convert legacy named mastery states to invented numeric scores.

Current Practice IPC exposes correctness/status/submission count and requested hints, not
these diagnostic, planning, or mastery engines. A presentation migration can ship first;
unsupported fields remain unavailable until backed by real persisted capabilities.

## 6. Invariants

1. Every screen gives the learner a clear next action and a way to inspect the reason.
2. Advanced policy and checker detail are optional disclosures; everyday copy stays readable.
3. Numeric mastery, charts, summaries, and clean-attempt counts describe real evidence and
   disclosed policies. A display never substitutes invented data for an unavailable capability.
4. Feedback names the issue and next action; red is allowed and never the sole signal.
5. Modules remain workspace-scoped capabilities. Stable product areas are not module names.
6. Visualization uses verified primitives shared across practice, tutoring, and notes.
7. Tutoring remains contextual; it does not become the permanent transcript of the app.
8. Resume restores the exact persisted attempt and working; a new plan never erases it.
9. One Guiding goal shapes recommendations; changing goals preserves history and previews effects.
10. The Reid theme is shared across all screens, expressed through centralized production tokens.
11. Domain scope is explicit; installing a capability never expands permanent navigation.
12. Offline operation and honest unavailable states are required; fresh installs contain no fiction.

## 7. Implementation order and open work

Roadmap Stage 9 schedules visual foundations, capability contracts, product navigation,
adaptive Practice/feedback, and Error profile/History. Preserve the Stage 8 regression and
offline acceptance work while migrating. Missing content families, tutor integration,
verification diagnostics, real visualization, permissions, marketplace downloads, and sync
still need their own scoped contracts; adopting a screen does not implement them.

Render and exercise the five adopted reference screens before fidelity approval. Test real
fresh launch, domain creation/switching, checking/hints, adaptive queue changes, profile
updates, history, and restart persistence as each capability lands. Verify native chrome
and reduced-transparency behavior on supported platforms.

## 8. Reference precedence

1. This handoff defines behavior, adoption boundaries, and cross-platform requirements.
2. The five October 2026 .dc.html files plus theme/component CSS define the visual target.
3. Legacy screenshots and the archived handoff describe prior behavior only where this
   document explicitly retains it; the old light/two-accent/no-red/no-score rules are superseded.
