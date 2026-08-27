# Axiom — UI Design Handoff

A complete description of the Axiom desktop learning environment as designed, written to be handed to another AI (or engineer) as the source of truth for implementation. Screenshots referenced below are in `screenshots/`.

Target runtime: **Tauri, multi-platform** (macOS, Windows, Linux). Native-desktop feel, but no platform-specific chrome, vibrancy, or traffic lights — the app leaves a 38px draggable strip at the top of its window and lets each OS supply its own decorations.

---

## 1. Product model (implementation-relevant)

```
Goal ──guides──> Workspace ──contains──> Concepts ──related to──> Concepts
                     │                        │
                     ├──enables──> Modules (tools)
                     └──records──> Sessions, notes, practice history, mastery
```

- **Workspace** — a curated learning environment for one subject (Calculus II). Owns goals, concepts, material, history, enabled modules, learning preferences. The chosen term over "Collection".
- **Goal** — a living object holding verbatim natural-language text plus inferred structure (deadline, mastery type, pacing, concept scope). One primary active goal per workspace; history retained.
- **Session intent** — a short-term goal ("practice 30 minutes", "understand shell integration"), distinct from the workspace goal.
- **Concept** — semantic node (Shell Method, Integration by Parts) with `prerequisite / related / leads-to` edges, a mastery state, and links to notes, practice, visualizations, tutor threads.
- **Module** — a capability enabled inside a workspace (Tutor, Visualizer, Practice, CAS, Notes, Review). Never a navigation destination; never described to users in developer language.
- **Mastery** — five named states: `New · Developing · Familiar · Strong · Mastered`. The **word is the indicator**; the ring glyph is shorthand only, and every ring appears within reading distance of its word. Never a percentage, never a chart. States decay — a workspace may honestly report "was Strong".
- **Offline** — a workspace-level promise. Resources and modules carry one of `Works offline · Online enhanced · Internet required`.
- **Goal states** — `Guiding · Waiting · Met · Resting`. Exactly one goal is Guiding and it alone shapes recommendations; a Waiting goal can take over on its own schedule.
- **Session intent** — the current activity ("Practising", "Reading", "Exploring") shown as a soft label with "Change intent". It steers; it never gates navigation.

---

## 2. Design system

### Surfaces (warm neutral, low chroma)

| Role | Value | Use |
|---|---|---|
| Content | `#FBFAF8` | main content panes, cards |
| Chrome | `#F2F1ED` | sidebar, toolbars, secondary panes (`#FAF9F6` for inner rails) |
| Recessed | `#E9E7E2` | wells, inactive fills |
| Ink | `#1C1B19` | primary text, dark buttons |
| Placeholder fill | `repeating-linear-gradient(135deg,#F2F0EA 0 7px,#E9E6DF 7px 14px)` | stands in for rendered visualizations/imagery |

Text tints: primary `#1C1B19`, secondary `rgba(28,27,25,.55–.62)`, metadata `rgba(28,27,25,.42–.5)`, hairlines `rgba(28,27,25,.07–.12)`.

### Accents — exactly two

- **Action** `oklch(.50 .13 258)` — primary buttons, links, active selection, progress fill, caret.
- **Mastery** `oklch(.50 .13 150)` — mastery glyph, verified badge, positive practice dots.
- Amber `oklch(.65 .12 60)` appears only as a "you made a specific mistake" dot. **No red anywhere.**

Both accents share lightness and chroma and differ only in hue, which keeps the palette flat and calm. No gradients on content (one 170° near-white gradient is allowed on the Continue card), no glow, no glass.

### Typography

System UI stack for interface (`ui-sans-serif, system-ui, "Segoe UI Variable Text", "Segoe UI", Roboto, "Helvetica Neue", sans-serif`), **STIX Two Text** for all mathematics — variables in italic, operators upright.

| Role | Size / weight | Notes |
|---|---|---|
| Screen title | 27 / 600, tracking −0.022em | one per screen |
| Section heading | 20–24 / 600 | |
| Card title | 15–17 / 600 | |
| Row title | 13–13.5 / 590 | |
| Body | 13.5 / 400, line-height 1.65 | `text-wrap: pretty`, max ~560px measure |
| Secondary body | 12.5 / 400, line-height 1.55 | |
| Metadata | 11.5 / 400 | |
| Eyebrow label | 10 / 600 monospace, uppercase, tracking .06em | "CONTINUE", "TOOLS" |
| Math inline | 13–17 STIX | |
| Math display | 20 STIX | centered in a `#F6F5F1` well |

Note for implementers: the shorthand `font: 400 12px/1 inherit` is **invalid CSS** and is dropped entirely — use longhands (`font-weight`/`font-size`/`line-height`) and let family inherit.

### Metrics

Sidebar 232px · window drag strip 38px · workspace toolbar 38px · session toolbar 44px · content gutter 32–44px · radii: rows 7–8, cards 11–14, sheets 16 · borders 1px hairline · shadows: `0 1px 2px rgba(28,27,25,.04)` for cards, `0 24px 60px rgba(28,27,25,.3)` for sheets. Density is "balanced".

### Components

- **Buttons** — primary (accent fill, white text, radius 8–9, 9–11px vertical padding), secondary (white, hairline border), tertiary (accent text only). Dark ink fill reserved for one-off emphasis (Browse Modules).
- **Segmented control** — `rgba(28,27,25,.07)` track, white selected pill with a 1px shadow. Used for learning modes (Explain / Practice / Reflect) and marketplace filters.
- **Chips** — inferred goal facets and concept tags; `display:inline-flex`, removable with a trailing ✕. Chips are objects: removing "Deadline · Dec 12" removes the deadline.
- **Mastery ring** — a 12–15px circle: hollow 1.5px border (New), then conic fill at 25 / 50 / 75 / 100% in the mastery accent (Developing → Mastered). Used identically on concept rows, concept view, workspace cards, command palette results, and material sections; always paired with the state word. Meanings: New = not started; Developing = can follow a worked solution; Familiar = solves typical problems with prompting; Strong = handles unfamiliar variants unaided; Mastered = held up weeks apart without review.
- **Chapter state profile** — up to five rings in a row on a collapsed group header, giving a chapter's mastery spread at a glance plus a plain count.
- **Reasoned recommendation** — an accent-ruled block: action, one line of evidence, a primary "Start · 8 min", and an optional **"Why this?"** that expands to dated observations. Never mentions AI, never explains the model.
- **Offline chips** — mint "Works offline", grey "Online enhanced", dashed amber-dot "Internet required". Shown only in marketplace listings, module detail, resource lists, and the workspace download sheet.
- **Progress bar** — 3px, unlabelled, never accompanied by a percentage.
- **Trust badge** — "Axiom Verified" (mint fill, mastery-hue text, dot) / "Community" (grey, may carry a human number like "4.8k learners") / "Experimental" (dashed outline).
- **Toggle** — 30×18 pill, accent when on, `rgba(28,27,25,.14)` when off.
- **Inspector / popover / sheet** — 95% white, hairline border, large soft shadow, always dismissible, always secondary to content.
- **Placeholder** — striped fill plus a monospace caption naming what belongs there.

### Copy rules

Concise, non-patronising, no exclamation marks, no emoji, no celebration. "Correct." not "Amazing!". Feedback names the misconception ("chose *u* backwards"), never just "incorrect". Reasons are stated in human terms ("you missed this twice on Tuesday"). Numbers appear only when they are context, never as scores.

---

## 3. Navigation

Chosen structure (study A of three explored — see `screenshots/13-navigation-studies.png`):

```
Search (⌘K)
Home
Marketplace
── WORKSPACES ─────────
Calculus II            ← open workspace expands
    Overview
    Concepts
    Material
    Tools
Linear Algebra
Circuit Analysis
+ New Workspace
── footer: avatar · name · settings
```

Rules: only the open workspace expands; the tree never exceeds two levels; modules are never rows in this sidebar. The sidebar is absent on first launch (nothing to navigate) and hidden only in Full Visualization mode, which offers a single "‹ Session" return.

Rejected alternatives, kept for reference: **B** 64px icon rail + header workspace switcher + segmented areas (max content width, weaker recognisability); **C** sidebar scoped to one workspace with its concepts and tools inline (deepest focus, harder cross-subject work).

---

## 4. Screens

### Screen 1 — First Launch · `screenshots/01-first-launch.png`
Centred column, max 520px, on a soft radial wash. Logo lockup (26px square outline with a rotated accent diamond) + "Axiom". H1 "What are you learning?", one sentence of reassurance, a single text field pre-filled with a ghost "Calculus II" plus **Continue**. Below a hairline: three text rows in ascending commitment — install a workspace template / import a syllabus, PDF, notes / explore a sample workspace. No sidebar, no account step, no tour, no mention of modules.

### Screen 2 — Create Workspace · `screenshots/02-create-workspace.png`
Two fields only: **Subject** (text) and **What are you trying to accomplish?** (textarea, natural language, focus ring visible). Below them a panel titled **"Axiom read that as"** showing inferred structure as removable chips (Deadline · Dec 12 / Mastery · conceptual, not just procedural / 14 concepts / Pacing · 4 sessions per week / Tools · Tutor, Practice, Visualizer, Notes) with an **Adjust** link that expands comfort level, materials drop target, and pacing in place. Footer: Create Workspace · Cancel · "Nothing here is permanent". Deadline and level are **never asked as form fields** — they are inferred and correctable.

### Screen 3 — Home · `screenshots/03-home.png`
Sidebar + content, max 800px measure. One line of remembered context ("Thursday afternoon · 3 days since your last session"). **Continue card**: eyebrow, "Calculus II — Shell Method", one sentence of exactly where you stopped, the half-finished integral typeset (`V = 2π∫₁³ x(x² − 1) dx`), Resume session / Open workspace, and a 206×150 thumbnail holding the visualization's last camera position. Then **Workspaces** — three cards, each carrying name, goal sentence, 3px unlabelled progress, last concept + relative time (a paused workspace says "Paused"). Footer text row: Templates · Marketplace · Import material. No dashboard, no analytics.

Alternates explored:
- **Session-intent led** (`03b`) — "How much time do you have?" with 15/30/60/browsing chips and a three-row plan sized to fit, each row stating its reason. Best for exam season and for freeze-at-the-start learners; presumes one active workspace. Could ship as a preference, not a fork.
- **Library led** (`03c`) — no sidebar; workspaces are the page, Continue shrinks to a resume strip with "Resume ⏎". Best for four or five simultaneous subjects; wasteful for one.

### Screen 4 — Workspace Overview · `screenshots/04-workspace-overview.png`
Answers "where am I, what next" in the first 300px. Title, goal sentence with "15 days out" and an **Edit goal** link. **Continue** card (concept, "Problem 3 of 5 · you were choosing the radius", Resume / Start something else). **Recommended next** as an accent-ruled block that states its own diagnosis: "You've set up shells correctly twice but picked the wrong radius when the axis wasn't x = 0." **Concepts in play** — four rows with human status ("active", "2 days ago", "due for review", "not started") and mastery glyphs, plus "All 14". Right rail 250px: four tool tiles (Tutor / Practice / Visualize / Notes) + "All tools & modules"; **Recent** as three bullets of real artefacts; and one grey suggestion panel proposing a goal change ("Series is next in the course… Add to plan").

### Screen 5 — Active Study Session · `screenshots/05-study-session.png`
Chosen layout: **stacked, visualization dominant** (of three studies — `screenshots/14-session-layout-studies.png`).

- Session toolbar (44px): concept name, subject line, **session-intent label** ("Practising · Shell Method · problem 3") with a "Change intent" link rather than mode tabs, five-dash session progress, "12′ of 30′", Pause. Learning modes are intent, not navigation: Axiom may nudge ("you've asked three questions about the same setup — want to see it instead?") but never blocks movement between explaining, practising, visualising and reflecting.
- Upper pane, `flex: 1.35` — the visualization. Floating verb toolbar (Rotate / Slice / Revolve) top-left, ⤢ and ⋯ top-right, and a bottom-left readout bound to the current selection: `radius r = x` · `height h = x² − 1` · "drag a shell to inspect".
- Lower-left pane (`flex: 1.55`) — "Problem 3 of 5", the prompt in prose with typeset symbols, the integral in a well **with the selected term `x` highlighted** and an "Ask about x" affordance, a dashed **YOUR WORKING** area containing the learner's own lines, then Check / Hint and "⌘⏎ to check".
- Lower-right pane — **Tutor · Socratic**: one diagnostic question about the visible object with three tappable answers, then a text field. A panel, not a chat log.
- Sidebar stays visible: a session is a mode of the workspace, not a separate window. Dividers are drag-resizable.

Rejected: problem-dominant with the visualization as a narrow inspector (good for solving, poor for intuition); immersive visualization with floating panels (striking, least stable).

### Screen 6 — Full Visualization Mode · `screenshots/06-full-visualization.png`
Full-bleed scene, **no sidebar**; header holds "‹ Session", the scene name, Save to notes, Share, Inspector. Bottom-centre floating toolbar with four verbs (Rotate / Slice / Revolve / Cross-section) and **More…**. Bottom-left **Bounds** panel: two labelled sliders (a, b), a shell count with a toggle, and **Advanced…**. Right **Selected shell** inspector — radius, height, `2πrh Δx ≈ 4.48`, one sentence of interpretation ("about 6% of the total volume; shells near x = 3 dominate"), Ask the tutor / Pin to notes; appears only on selection and is dismissible. Zoom and recentre controls bottom-right.

Visualization philosophy to preserve: scenes are **composed from verified primitives** (coordinate system, function, region, axis, revolution, shell, annotation), not generated as images. The inspector reads out those primitives, and the same object is shared by tutor, practice, notes, and assessment.

### Screen 7 — Concept View · `screenshots/07-concept-view.png`
Breadcrumb Calculus II › Concepts › Integration by Parts. Title, then mastery as **ring plus state name and its meaning** — "Mastered — held up weeks apart" — beside "Due for review in 2 days". Display formula `∫ u dv = uv − ∫ v du` in a well. Two paragraphs: the concept explained in the product's own voice, then the learner's own heuristic quoted back at them with evidence ("held up in four of your last five problems"). Verbs: Practice this / Ask the tutor / **Explain it back** (Reflect gets equal billing). "Where it shows up" as tags. Right rail: **Builds on** and **Leads to** with mastery glyphs + "See in concept map"; **Recent practice** typeset and diagnostic (`∫ eˣ sin x dx` — chose *u* backwards, amber dot); **Your notes** excerpt + "2 more notes".

### Screen 8 — Workspace Tools · `screenshots/08-workspace-tools.png`
Title "Tools in this workspace" and one reassurance: "Turn something off and it disappears from the workspace — nothing you've made with it is deleted." **On** — six rows (Axiom Tutor, Mathematical Visualizer, Practice, CAS, Notes, Review), each with icon, name, trust badge where relevant, a line saying what it does for learning and what context it sees, Settings, and a toggle. **Off** — three compact tiles (Exam Simulator, Flashcards, Concept Map). Bottom: one suggestion justified by observed behaviour ("Suggested because you keep asking the tutor *why* a method works") with Not now / Take a look. Language is On/Off, never install/uninstall, versions, dependencies, or manifests.

### Screen 9 — Module Marketplace · `screenshots/09-marketplace.png`
Sidebar gains Marketplace sub-items (For you / Templates / Categories / Installed). Opens on **"For your Calculus II workspace"**. Hero row: one featured module with a preview strip, verified badge, educational description, Install / Learn more, "Axiom Labs · Free"; beside it two **Workspace Templates** (Visual Learner, Exam Intensive) each stating its tool count. Then a **Modules** grid of three (Proof Assistant — verified; Series Intuition Pack — community, 4.8k learners; Quiet Mode — community, accessibility), categories as a quiet text row. Bottom: a dashed "Load local module" row for tinkerers — present, unpromoted, not alarming. Descriptions state learning value; no ratings, no marketing language, no warnings on community items.

### Screen 10 — Module Detail · `screenshots/10-module-detail.png`
Icon, title, verified badge, "Axiom Labs · Free · Updated last week". Actions: **Install to Calculus II** (install is workspace-scoped), **Try it first** (live sandboxed preview), "Add to another workspace". A 220px live interactive preview plus four thumbnails. **What it adds to your learning** — two paragraphs, the second being the verified-primitives promise in plain language. Supported concepts as tags. Right rail: **What it can see** — four sentences with defaults visible ("Your notes — off by default", "Nothing leaves your device") and "Change what it sees"; **Works with** (Tutor, Practice, Notes · Review); **Suits** (courses and learning style); three metadata rows (developer, learners using it, offline). Permissions are phrased as capability, never as a scary list.

### Screen 11 — Goal Editing · `screenshots/11-goal-editing.png`
A 560px **sheet over the dimmed workspace**, not a settings page. Eyebrow "Calculus II · Primary goal", H2 "What are you working toward?", the same natural-language field as Screen 2 containing the new goal, and under it "Was: 'Pass Calculus II.'" with **Revert**. Inferred facets as removable chips (Deadline · none / Mastery · conceptual / Pacing · steady / + Add). Then the key panel, **What changes** — four bullets previewing consequences in learning terms, ending with "**Nothing is deleted.** Notes, mastery, history, and this workspace's previous goal are kept." Footer: Update goal · Cancel · Goal history.

### Screen 12 — Command Palette · `screenshots/12-command-palette.png`
600px overlay, 96px from the top of the window, over dimmed content. Query "shell", workspace scope badge top-right. Four result groups: **Actions** (Practice the Shell Method — top hit, accent-tinted, with the consequence "5 problems · adaptive" and ⏎; Visualize shells about x = 0; Ask the tutor ⌘T; New note ⌘N), **Concepts** (with inline mastery glyphs and status), **From your work** (a note), and one marketplace result carrying a Community badge. Footer key legend: ↑↓ move · ⏎ run · ⇥ scope · esc close. Commands are phrased as things to do, never as identifiers. ⌘K is the only shortcut a beginner needs, and the sidebar advertises it.

---

## 5. Scale behaviour (pressure-tested)

The same layouts were tested against a semester of real content. These behaviours are part of the design, not later optimisations.

### Returning after time away · `screenshots/16-returning-after-time-away.png`
After a long absence the Continue card is **replaced** (same slot, same weight) by context recovery: "You were setting up shell-method integrals", three lines tied to mastery states — what held, what didn't, what changed underneath you — then two exits: a 5-minute refresher or straight back to the exact problem. A **Faded while away** rail names decay honestly ("Trig Substitution · was Strong"). "While you were away" is three dated lines and then stops; a semester never becomes a feed.

### 87 concepts · `screenshots/17-concepts-at-scale.png`
The list never opens flat. **Needs work** first (six, three shown), ordered by what it blocks. Everything else collapses **by chapter** — matching the textbook's mental model — with a five-ring state profile and a plain count per chapter. Filters are actionable counts ("Due for review · 11", "On the exam · 22"), not taxonomy. Graph is a toolbar toggle, never the default; the selection rail shows the prerequisite **chain**, which covers the 90% case, plus attached material, visualizations, notes, and problem-bank counts.

### 712-page textbook · `screenshots/18-material-textbook.png`
Material is reached through concepts and search — **there is no folder view**. Results are typed (section / worked example / exercise range), each carrying the reason it matters to this learner and the concept + mastery state it belongs to. "Where you are in the book" compresses 18 chapters into a four-segment bar; out-of-syllabus chapters stay available but never appear in recommendations. Highlights surface as a count plus a "most marked" line. Reading in-app pins tutor answers to the passage and files them under the concept.

### Long session, 40 tutor exchanges · `screenshots/19-long-session-tutor.png`
The tutor panel never becomes a transcript. Exchanges collapse into **"What we've settled"** — two conclusions and one open question, promotable to concept notes — with the full history behind "Earlier today". Only the current exchange is shown in full, so panel height is constant. Explanations can be **pinned to the object** as annotations on the visualization. The sidebar auto-collapses to a 56px rail during deep focus (restore with » or ⌘0). A break suggestion appears once, in the tutor's voice, protective rather than nagging.

### Large visualization · `screenshots/20-learning-canvas.png`
The learning canvas is the one place Axiom **inverts its own palette**: dark stage, luminous curve, translucent shells, real depth. The chrome is restrained everywhere else precisely so this can be expressive. The equation sits under the object with the dragged term tinted the same hue as the geometry it controls (selection is bidirectional). Five controls visible; bounds, shell count, animation and rendering appear on ⌥ or from More…. The tutor is one corner line plus a "watching this" marker. "Back to problem" keeps it a detour inside the session, not a destination.

### Offline, 20 modules, 4 goals · `screenshots/21-offline-modules-goals.png`
**Make available offline** is one workspace sheet: four per-kind toggles, honest sizes, one total, and a plain statement of what degrades rather than breaks ("typed tutoring keeps working, with shorter answers"). Third-party limits are stated in the learner's terms. Once downloaded, a single "Available offline" chip sits in the workspace toolbar and nothing else mentions connectivity.

**Twenty installed modules** group by *where they appear*: In the workspace · 4 (the Overview tiles) / Appear when relevant · 9 / Off in this workspace · 7. Axiom hides a tool unused for six weeks and says so once. **No module ever adds a sidebar row** — the four workspace areas are the permanent maximum.

**Four goals**: one Guiding (the only one shaping recommendations), Waiting ones that can take over on their own schedule, Met ones archived with what they achieved, Resting ones collapsed. Overview shows only the guiding goal with "+2 more".

---

## 6. Invariants to preserve

1. Simple by default, powerful by choice — every screen shows one obvious next action; advanced controls sit behind Adjust / Advanced… / More… / All tools.
2. Never expose configuration merely because it exists; never use developer vocabulary (package, manifest, dependency, runtime, API) in the learner interface.
3. No dashboards, no percentages, no streaks, XP, badges, or leaderboards. Progress is expressed as named mastery states and capability sentences.
4. Feedback diagnoses; it never merely marks. Amber for a named mistake; no red; no celebration.
5. Modules are capabilities inside a workspace, never navigation destinations, and they share workspace context.
6. Visualization is composed from verified primitives and shared across tutor, practice, explanation, notes, assessment.
7. The tutor is a contextual panel that already knows what you are looking at — Axiom is not a chatbot.
8. A learner returning after three days must never have to reconstruct what they were doing: the Continue surfaces do it for them.
9. Goals are living objects; changing one previews its consequences and destroys nothing. Exactly one goal guides at a time.
10. Two accents, two type families, one placeholder treatment. Any new surface must be expressible in that vocabulary — the dark learning canvas is the single sanctioned exception, and only for the mathematical object itself.
11. Permanent navigation never grows with capability. Modules surface contextually; the sidebar answers "where am I?" and nothing more.
12. Offline is a promise, surfaced only where it changes a decision.

---

## 7. Not yet designed

Empty states, notifications and menus, the full concept-graph view, tutor voice mode, in-app reading mode (the page view behind Material), settings, developer-facing module authoring, onboarding for imported material, and responsive behaviour below ~1100px window width.

## 8. Screenshot index

`00-foundations` · `01-first-launch` · `02-create-workspace` · `03-home` (+ `03b` session-intent and `03c` library alternates) · `04-workspace-overview` · `05-study-session` · `06-full-visualization` · `07-concept-view` · `08-workspace-tools` · `09-marketplace` · `10-module-detail` · `11-goal-editing` · `12-command-palette` · `13-navigation-studies` · `14-session-layout-studies` · `15-system-refinements` (current mastery / recommendation / offline / intent components) · `16-returning-after-time-away` · `17-concepts-at-scale` · `18-material-textbook` · `19-long-session-tutor` · `20-learning-canvas` · `21-offline-modules-goals`.

Screens 5, 6, 9 and 10 (`05`, `06`, `09`, `10`) predate the five-state mastery language and the session-intent label; where they show the older segmented control or bar glyph, **`15-system-refinements` is authoritative**.
