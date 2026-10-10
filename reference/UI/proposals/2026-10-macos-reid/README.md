# Adopted direction: macOS-style Axiom UI on Reid Engineered

**Status: full visual and product direction adopted on 2026-10-10.** The human explicitly
selected full adoption after PR #25 merged. The amended
[UI handoff](../../AXIOM-HANDOFF.md) is the implementation authority; task 081 and
[the decision](../../../../docs/superpowers/specs/2026-10-10-reid-design-adoption.md) record
the change. Sample values and inert controls remain illustrations, not runtime evidence.
The original independent review is archived under .ai/tasks/_archive/080-macos-ui-proposal-review.md.

Drawn 2026-10-09 in a Claude design canvas from the Reid Engineered design system (the
shared look for Marcus's Mac desktop tools, Meridian first). All data is sample data on
MTH 112 topics. Math is hand-set spans; the app would use the handoff's math stack.

## Files

| File | Screen |
|---|---|
| `Main.dc.html` | Today: session plan with named phases, difficulty, watched patterns, fading skills, concept map |
| `Session.dc.html` | Active question with a "Why this question" inspector |
| `Feedback.dc.html` | After a miss: names the misconception, shows what the miss changed, swaps the next question |
| `Profile.dc.html` | Error profile: recurring / improving / resolved patterns, pattern detail inspector |
| `Summary.dc.html` | Session summary: question-by-question results, profile changes, next session |
| `axiom-theme.css` | Reid Engineered dark values with its `alt-blue` mapped onto `accent` |
| `ds/reid/components/bundle.css` | Reid Engineered component CSS (`re-` classes), copied unmodified |

Open the `.dc.html` files the same way as `reference/UI/Axiom.dc.html` (they load
`../../support.js`). Markup inside `<x-dc>` is plain HTML; ignore the `<helmet>` and
`data-dc-script` wrappers.

## What the proposal is trying to add

These are the ideas worth judging separately from the visual style:

1. **The error profile as a first-class screen.** Recurring misconceptions named in the
   learner's terms, with evidence ("6 of your last 14 indefinite integrals"), a status, and
   an explicit rule for when one counts as resolved.
2. **"Why this question" during practice.** The inspector says why the engine picked this
   problem and which of the learner's own patterns it is testing.
3. **A miss that changes something visible.** Feedback names the misconception, links it to
   the profile, and shows that the next question was swapped to test it again.
4. **A session with a stated shape.** Warm-up / stretch / combined, each with a one-line
   reason drawn from the learner's history, and a difficulty setting with stated stakes.

## Historical conflicts with the superseded handoff

This table records conflicts with the former handoff. The human-approved amendment
supersedes those visual/progress/navigation prohibitions; it retains truthful data, goal
continuity, real platform controls and capability isolation. Refer to the current handoff
for the adopted behavior rather than treating this historical table as a blocking rule.

| Proposal does | Handoff says | Where |
|---|---|---|
| Traffic lights, blurred floating sidebar, overlay title bar | No platform-specific chrome, vibrancy or traffic lights; multi-platform; 38px drag strip | Intro, §2 |
| Mastery as numbers (0.62) and meters with a decay "ghost" | Five named states; the word is the indicator; never a percentage, never a chart; decay shown as "was Strong" | §1, §2 Mastery ring, invariant 3 |
| Stat tiles, a per-session bar chart, a results table with deltas | No dashboards, no percentages; numbers only as context, never as scores | §2 Copy, invariant 3 |
| Red (`danger`) for misses and regressions | No red anywhere; amber dot for a named mistake | §2, invariant 4 |
| Dark-first everywhere | Warm light surfaces; dark only for the learning canvas | §2 Surfaces, invariant 10 |
| One accent (blue) | Two accents: action and mastery | invariant 10 |
| New York serif for titles and math | System UI for interface, STIX Two Text for all math | §2 Type |
| Status words "Incorrect" / "Correct" | Feedback names the misconception, never just "incorrect" | §2 Copy, invariant 4 |
| Sidebar of app scopes (Today, Practice, Error profile…) with a domain list | Sidebar is the workspace tree, two levels, no modules as rows | §3 |
| A "Next exam" card | Deadlines live on the goal as an inferred, removable facet | Screen 2, Goal editing |

All four ideas and the visual direction are now adopted. Implementation is sequenced
in Roadmap Stage 9 and tasks 082–086; the old recommendation to redraw into the former
palette/navigation vocabulary is superseded.

## Adopted mechanics requiring implementation contracts

The direction is approved. Task 083 must lock actual policies, payloads and persistence;
the numeric examples below are candidate values rather than validated calibration:

- Pattern resolution rule (mock: 5 clean attempts in a row on questions that could trigger it).
- Swapping the next question after a miss so the same misconception is tested again.
- Difficulty profiles (Steady / Push / Exam) with a hint budget and miss cost from config.
- Per-template attribution of which misconceptions a problem can trigger.
- Exposing checker detail to the UI (samples, domain, which term differs).

## Data the screens would need from the core

For the reviewer to check against `ARCHITECTURE.md` / `CORE.md`, not to implement here:
per-concept mastery state plus its previous state and last-practised date; misconception
records (occurrences over opportunities in a window, first/last seen, clean run, status);
a session plan with named phases and a reason per phase; per-question misconception
attribution; difficulty profile from config; checker result detail. Calculus-specific
strings (concept names, "+ C", triangle rules) must come from the knowledge package, never
from the engine or shell components.
