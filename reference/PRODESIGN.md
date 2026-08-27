# Axiom — Product Design Brief

You are a senior product designer specializing in **macOS-quality desktop applications, educational software, information architecture, and interaction design**.

Design **Axiom**, a modular, adaptive desktop learning environment.

Axiom should feel as polished, calm, intentional, and cohesive as a high-quality native macOS application, while remaining visually distinct and recognizable as its own product.

Use the principles and spirit of the **Apple Human Interface Guidelines** as a reference for usability, hierarchy, navigation, spacing, typography, interaction states, progressive disclosure, accessibility, and platform conventions.

Do **not** simply copy Apple's applications.

The result should feel like:

> "This belongs on macOS."

rather than:

> "This is a clone of an Apple app."

---

# 1. Product Vision

Axiom exists because people do not all learn in the same way.

Traditional educational software generally provides a predetermined learning experience and expects the learner to adapt to it.

Axiom reverses that relationship.

**Axiom should adapt to the learner.**

It is not simply an AI tutor, flashcard application, note-taking application, graphing calculator, or course platform.

It is better thought of as a:

## Modular Learning OS

Axiom provides the environment, foundational learning systems, shared context, and tools required for studying.

Users then shape that environment around how they personally learn.

The ultimate goal is to allow two people studying the exact same subject to construct very different learning environments while still using the same underlying application.

---

# 2. Core Philosophy

The primary design principle of Axiom is:

# Simple by default. Powerful by choice.

This principle should influence every interface decision.

A new user should be able to install Axiom, open a workspace, and begin studying almost immediately.

An advanced user should be able to deeply customize that same environment with different tools, modules, visualization systems, tutors, workflows, and learning strategies.

Complexity should be revealed progressively.

Never expose configuration simply because configuration exists.

The product should reward curiosity without requiring technical knowledge.

---

# 3. Product Mental Model

The important conceptual hierarchy is:

**Vision → Philosophy → Workspaces → Goals → Concepts → Modules**

These are conceptual relationships rather than necessarily literal navigation levels.

Avoid turning this hierarchy into a complicated folder tree.

---

# 4. Goals

Every workspace should have a reason for existing.

A user should be able to define a high-level learning goal when creating or modifying a workspace.

Examples:

- "Pass my Calculus II final."

- "Build deep intuition for linear algebra."

- "Learn enough circuit analysis to succeed in my electronics course."

- "Prepare for Exam 2."

- "Become fluent with integration techniques."

- "Learn this material without a deadline."

- "Review everything I forgot from Calculus I."

Goals should feel natural and human.

Users should preferably be able to describe a goal in normal language rather than filling out a complex form.

Axiom can infer additional structure when appropriate, including:

- desired mastery

- deadline

- relevant concepts

- current knowledge

- pacing

- recommended study strategy

However, the UI should clearly distinguish between:

## Workspace Goal

The long-term destination.

Example:

"Master Calculus II."

and

## Session Intent

What the learner wants to accomplish right now.

Examples:

- Practice for 30 minutes

- Understand shell integration

- Review weak concepts

- Prepare for tomorrow's quiz

- Explore visually

- Work through difficult problems

Goals are **living objects**, not static labels.

They can evolve without requiring the user to create a completely new workspace.

A workspace may contain multiple goals over its lifetime, but should normally have one primary active goal guiding recommendations.

---

# 5. Workspaces / Collections

The terms **Workspace** and **Collection** are currently interchangeable during product development.

For the interface, prefer whichever name creates the clearest user experience.

A workspace is a **curated learning environment centered around a subject, goal, or area of study**.

Examples:

- Calculus II

- Linear Algebra

- Circuit Analysis

- Chemistry

- Signals and Systems

- Japanese

- Physics

A workspace is more than a folder.

It contains and connects:

- goals

- concepts

- study history

- learning progress

- resources

- notes

- problems

- visualizations

- tutor context

- enabled modules

- assessment data

- learning preferences

A workspace should feel **alive**.

When the learner returns, the workspace should understand approximately:

- what they have studied

- what they understand

- what they struggle with

- what they were doing previously

- what they should probably do next

The workspace should answer:

> "What should I study now?"

without forcing the learner through a complicated dashboard.

---

# 6. Concepts

Concepts are the semantic backbone of learning inside Axiom.

Examples inside Calculus II might include:

- Integration by Parts

- Disk Method

- Washer Method

- Shell Method

- Parametric Curves

- Sequences

- Infinite Series

- Taylor Series

Do not treat knowledge as merely files inside folders.

Resources, problems, tutor conversations, visualizations, notes, and assessments may all relate to one or more concepts.

Concepts can have relationships such as:

- prerequisite

- related concept

- depends on

- applied by

- extension of

This creates the basis of a **learning graph**.

The learning graph does not need to dominate the UI.

It should exist as a powerful underlying model that can surface visually when useful.

---

# 7. Modules

Modules are the main mechanism that makes Axiom customizable.

A module is a reusable learning capability that can be added to a workspace.

Examples may include:

- AI Tutor

- Problem Generator

- Practice Engine

- Flashcards

- Spaced Repetition

- Mathematical Visualizer

- Graphing Tool

- CAS / Symbolic Math

- Equation Solver

- Proof Assistant

- Exam Simulator

- Concept Map

- Notebook

- Derivation Explorer

- Memory Trainer

- Reading Assistant

- Formula Reference

- Coding Sandbox

- Chemistry Visualizer

- Circuit Simulator

- Timeline Explorer

- Language Conversation Partner

Modules should not feel like completely separate applications.

They should feel like capabilities available inside the same learning environment.

All modules should share the workspace's learning context when appropriate.

For example:

A learner struggles with the Shell Method.

The Practice module detects difficulty.

The workspace updates mastery information.

The Tutor recognizes the misconception.

The Visualizer can display the exact solid currently being discussed.

The Review module can later schedule another related problem.

These tools should feel coordinated even though they are modular.

---

# 8. Module Marketplace

Axiom should eventually have a marketplace or module library.

Users should be able to extend Axiom by installing modules created by:

- Axiom

- verified developers

- educational organizations

- teachers

- community developers

- individual users

However:

**Do not make Axiom look like a developer tool.**

The average learner should not need to understand modules technically.

The default interaction should resemble:

1. Discover something useful.

2. Install it.

3. Use it.

Advanced configuration should exist behind progressive disclosure.

Modules may have trust indicators such as:

- Axiom Verified

- Community

- Experimental

- Local / Custom

Do not aggressively discourage unofficial experimentation.

Axiom should embrace the philosophy that users can shape their own environment.

---

# 9. Workspace Templates

Users should not need to construct every workspace manually.

A marketplace item may also be a complete **Workspace Template**.

Example:

## Calculus II — Visual Learner

Could include:

- AI Tutor

- CAS

- 2D Graphing

- 3D Visualization

- Practice Generator

- Spaced Review

- Concept Graph

Another template might be:

## Calculus II — Exam Intensive

containing:

- Exam Simulator

- Timed Practice

- Weakness Detection

- Formula Review

- Problem Generator

The user installs the environment and can immediately begin studying.

They can customize it later.

Again:

**Simple by default. Powerful by choice.**

---

# 10. Learning Experience

Do not reduce studying to endlessly answering generic questions.

Axiom should support a richer learning loop:

## Discover

Encounter or introduce a concept.

Possible experiences:

- explanation

- textbook material

- video

- lecture notes

- tutor introduction

## Understand

Build an internal mental model.

Possible experiences:

- tutor explanation

- examples

- analogies

- visualization

- derivations

- concept relationships

## Practice

Test understanding.

Possible experiences:

- generated problems

- selected problems

- conceptual questions

- calculations

- guided practice

## Apply

Use knowledge in unfamiliar situations.

Possible experiences:

- challenging problems

- projects

- cross-topic questions

- engineering applications

- real-world scenarios

## Reflect

Force active understanding.

Examples:

- explain the concept yourself

- identify why a solution works

- predict what happens before calculating

- compare two methods

- identify your mistake

## Review

Revisit material intelligently based on:

- demonstrated mastery

- forgetting

- previous mistakes

- importance

- upcoming deadlines

These are **learning modes**, not necessarily literal tabs.

---

# 11. Adaptive Questioning

Axiom should explicitly avoid the repetitive, generic feeling common in AI study applications.

Questions should attempt to diagnose understanding.

A learner getting something wrong should not simply receive:

> "Incorrect. Try again."

Instead, Axiom should reason about what misconception may have caused the mistake.

Question difficulty should evolve based on demonstrated ability.

Questions can test different dimensions:

- recall

- conceptual understanding

- procedural fluency

- transfer

- reasoning

- prediction

- explanation

- error identification

A correct answer should not automatically mean mastery.

Axiom should distinguish between:

- guessed correctly

- mechanically solved

- conceptually understood

- independently applied

- consistently mastered

Avoid presenting these distinctions as excessive analytics unless the learner requests them.

---

# 12. AI Tutor

The AI Tutor should be a first-class module, but **Axiom itself is not merely a chatbot**.

Avoid making a giant chat window the central product experience.

The tutor should behave more like a knowledgeable person studying beside the learner.

It should understand relevant workspace context, such as:

- current concept

- current problem

- visible equation

- active visualization

- notes

- learning goal

- mastery state

- recent mistakes

Imagine:

> "A tutor sitting at the same desk and looking at the same material."

rather than:

> "A chatbot waiting for you to explain everything."

The tutor could support interaction modes such as:

- Teacher

- Coach

- Socratic Tutor

- Study Partner

- Exam Proctor

These should modify behavior, not necessarily be presented as gimmicky personalities.

Voice tutoring may exist as an optional capability.

A voice tutor should be capable of discussing what the learner is currently viewing or manipulating.

Voice must remain optional.

---

# 13. Visualization as a Core Strength

Visualization should be one of Axiom's defining capabilities.

Axiom should not merely display static diagrams.

Learners should be able to **manipulate concepts**.

Examples:

- rotate a solid

- drag a point

- move a tangent

- adjust bounds

- change a parameter

- animate a limit

- visualize a derivative

- watch a Riemann sum converge

- slice a solid

- revolve a region

- inspect cross sections

- manipulate vectors

- transform matrices

- change basis

- visualize electric fields

- inspect circuit behavior

The design should make visualization feel deeply integrated into learning.

A visualization may be used simultaneously by:

- tutor

- practice module

- explanation

- assessment

- notebook

---

# 14. Verified Visualization Philosophy

Do not assume AI should freely generate mathematical diagrams.

Axiom should conceptually treat visualization more like **compilation than image generation**.

The conceptual flow is:

Natural language or learning intent

→ visualization intent

→ structured visualization plan

→ trusted mathematical primitives

→ verified scene

→ renderer

The visual system should be built from reusable, reliable primitives.

Examples of primitives include:

### Coordinate Systems

- 2D Cartesian

- 3D Cartesian

- polar

- parametric

- number line

### Mathematical Objects

- point

- line

- curve

- function

- vector

- plane

- region

- surface

- volume

- matrix

### Operations

- rotate

- revolve

- slice

- translate

- intersect

- project

- transform

- differentiate

- integrate

- approximate

### Educational Objects

- tangent line

- secant line

- Riemann rectangles

- shell

- washer

- disk

- cross section

- gradient

- vector field

- eigenvector

- transformation grid

### Presentation Components

- annotation

- label

- highlight

- measurement

- step

- animation

- focus region

Complex educational visuals should be **compositions of trusted primitives**.

For example:

A washer-method visualization should not need to be manually built as a unique animation every time.

It can be composed from:

- function

- region

- axis

- revolution

- slice

- washer

- annotation

- animation

This creates visual consistency and makes AI-assisted generation substantially safer and more predictable.

---

# 15. Human Supervision Philosophy

AI tools may help build Axiom and may eventually help construct educational material.

However, the human developer should operate primarily as a **high-level supervisor of outputs built from verified foundations**.

Do not design workflows that require manually validating thousands of arbitrary generated diagrams.

Instead, make foundational systems reliable enough that composition produces predictable results.

The product philosophy should favor:

- deterministic foundations

- reusable components

- verified primitives

- reusable learning recipes

- composability

- automated validation

- human review of foundations

over:

- unrestricted AI generation

- one-off hand-authored experiences

- giant libraries of manually built examples

---

# 16. First Launch Experience

Design a thoughtful first-launch experience.

Do not immediately overwhelm users with:

- marketplace

- module configuration

- graphs

- settings

- dashboards

- analytics

Instead, begin with something understandable.

Potential structure:

## Welcome to Axiom

"What are you learning?"

The user can:

- create a workspace

- import something

- install a workspace template

- explore a sample workspace

When creating a workspace, Axiom can ask:

### What are you learning?

Example:

Calculus II

### What are you trying to accomplish?

Example:

"I want to deeply understand the material and pass my final in December."

Potential optional questions:

- When is your deadline?

- What material are you using?

- How comfortable are you already?

- How do you prefer to study?

These should not feel like mandatory onboarding forms.

The user should be able to skip most configuration and begin.

---

# 17. Home Screen

The home screen should be calm.

Avoid a giant analytics dashboard.

The user should immediately see their learning environments and know where to continue.

Possible elements:

## Continue

Show the most relevant active workspace and previous context.

Example:

**Calculus II**

Shell Method

Continue studying

## Workspaces

A clean collection of the user's subjects.

Potential workspace cards may subtly display:

- name

- active goal

- recent concept

- progress

- next suggested action

Do not overload cards with statistics.

Potential secondary elements:

- New Workspace

- Templates

- Recent

- Marketplace

These should not compete with the primary action:

**Continue learning.**

---

# 18. Workspace Home

When opening a workspace, the learner should not land on an empty document.

The workspace should answer:

> "Where am I, and what should I do next?"

Possible structure:

### Header

Calculus II

Goal:
Master the course and prepare for final exam

### Continue

Resume:
Shell Method — Solids of Revolution

### Suggested Next Action

"Try three shell-method problems focused on choosing the correct radius."

### Current Concepts

A lightweight representation of recent or important concepts.

### Workspace Tools

Only frequently used modules should be immediately visible.

Examples:

Tutor
Practice
Visualize
Notes

Everything else may live behind a modules/tools control.

The screen should remain calm.

---

# 19. Study Session

Consider a focused study-session interface.

A study session might contain:

- current concept

- current activity

- contextual tutor

- visualization

- problem area

- optional resources

- progress through the current session

Avoid making the user switch constantly between disconnected pages.

The UI should allow the environment to reconfigure around the learning activity.

For example:

### During Visual Exploration

Visualization becomes dominant.

Tutor becomes contextual.

Notes remain accessible.

### During Problem Solving

Problem becomes dominant.

Scratch/workspace area becomes available.

Tutor remains available but unobtrusive.

Visualization can appear beside the problem when useful.

### During Reading

Content becomes dominant.

Annotations and tutor become contextual.

The layout should adapt to intent without feeling unstable.

---

# 20. Module Management

There should be a clear place where users can see which modules are installed in a workspace.

However, this should feel more like customizing a Mac application than configuring a development environment.

Potential presentation:

## Workspace Tools

Installed:

- Tutor

- Practice

- Visualizer

- Notes

- Review

Available:

Browse Modules

Each module may expose optional settings.

Advanced settings should be hidden until requested.

Avoid technical terminology such as:

- dependency graph

- API

- runtime

- package

- manifest

in the standard learner interface.

Those concepts may exist in a separate developer experience later.

---

# 21. Marketplace

Design the marketplace as an extension of the learning environment.

Possible categories:

- Mathematics

- Science

- Engineering

- Languages

- Writing

- Programming

- Memory

- Productivity

- Accessibility

And module types such as:

- Tutors

- Visualizers

- Practice

- Assessment

- Notes

- Simulators

- Reference

- Workspace Templates

Marketplace pages should clearly communicate:

- what the module does

- how it helps learning

- what information it accesses

- whether it is verified

- screenshots / previews

- compatible subjects or workflows

Prioritize educational usefulness over app-store-style marketing.

---

# 22. Progressive Disclosure

Axiom may eventually become extremely powerful.

The interface should not reveal that complexity simultaneously.

Use progressive disclosure aggressively.

For example:

A beginner sees:

**Visualize**

An advanced user may eventually discover:

- coordinate settings

- symbolic parameters

- animation controls

- rendering options

- module configuration

- custom primitives

The beginner should never need to encounter these controls.

---

# 23. Visual Design Direction

Axiom should feel:

- calm

- intelligent

- precise

- premium

- focused

- modern

- spatial

- slightly scientific

- approachable

Avoid:

- childish gamification

- excessive gradients

- glowing AI effects

- neon everywhere

- generic SaaS dashboards

- excessive rounded cards

- giant empty hero sections

- constant chat bubbles

- dense analytics

- "AI magic" visual clichés

- overly futuristic interfaces

- excessive glassmorphism

Use depth carefully.

macOS-inspired materials may be appropriate for:

- sidebars

- floating inspectors

- popovers

- tool palettes

- contextual panels

but content surfaces should remain highly readable.

---

# 24. Typography

Typography should feel native to macOS.

Prefer system typography principles similar to San Francisco:

- strong typographic hierarchy

- excellent readability

- restrained number of sizes

- clear distinction between navigation, title, body, metadata, and mathematical content

Mathematical notation should receive special attention.

Equations should feel like first-class content, not pasted images.

---

# 25. Navigation

Prefer a familiar desktop hierarchy.

Potential structure:

### Sidebar

Home

Workspaces

---

Calculus II
Circuit Analysis
Linear Algebra

---

Marketplace

Optional lower items:

Settings
Profile

Inside a workspace, navigation may change contextually.

Potential workspace areas:

Overview
Learn
Concepts
Resources

But avoid creating unnecessary permanent tabs.

Modules should preferably surface when relevant rather than each becoming a navigation destination.

---

# 26. Inspectors and Contextual UI

Use contextual inspectors where appropriate.

For example, selecting a mathematical object might reveal:

- equation

- parameters

- visual settings

- educational annotations

Selecting a concept might reveal:

- mastery

- related concepts

- resources

- recent mistakes

- recommended activities

These panels should remain dismissible and secondary to the learning content.

---

# 27. Command and Power-User Features

Axiom should eventually reward advanced users.

Consider a command palette similar to powerful desktop productivity applications.

Possible commands:

- Open Workspace

- Start Practice Session

- Visualize Current Concept

- Ask Tutor

- Create Note

- Install Module

- Switch Goal

- Review Weak Concepts

Keyboard shortcuts should be considered.

However, these should never be required for normal usage.

---

# 28. Accessibility and Different Learning Styles

Axiom exists specifically because learners differ.

Accessibility should therefore be foundational rather than an afterthought.

Consider:

- reduced motion

- high contrast

- keyboard navigation

- screen readers

- adjustable text size

- visual density options

- distraction reduction

- optional voice interactions

- alternative representations of concepts

- configurable session length

The interface should permit different levels of stimulation.

Some learners may want an extremely quiet environment.

Others may benefit from:

- timers

- visible progress

- frequent interaction

- animation

- active tutoring

Do not assume one mode is universally superior.

---

# 29. ADHD-Friendly Design Without Creating an "ADHD Mode"

The product should naturally support users who struggle with attention, executive function, or overwhelming interfaces.

Do not stigmatize this with a giant dedicated mode.

Instead apply good design:

- obvious next action

- limited simultaneous choices

- resumable sessions

- visible context

- minimal navigation overhead

- short achievable study intents

- optional focus sessions

- low-friction capture

- easy recovery after leaving the app

A user returning after three days should not need to reconstruct what they were doing.

Axiom should tell them.

---

# 30. Avoid Gamification as the Primary Motivation System

Do not turn Axiom into a game.

Avoid making streaks, XP, badges, coins, or leaderboards the dominant motivational system.

Meaningful progress should come from:

- understanding

- competence

- completed goals

- mastery

- successful application

Subtle motivational elements are acceptable if they support learning rather than manipulate engagement.

---

# 31. Product Personality

Axiom should behave like an excellent teacher and an excellent tool.

It should be:

- confident but not arrogant

- concise

- curious

- supportive

- intellectually serious

- non-patronizing

Avoid excessive celebration.

Do not display:

"AMAZING!!! 🎉🔥"

because a learner solved a routine equation.

Prefer understated feedback such as:

"Correct."

or:

"You've got the method. Let's make the next one less obvious."

---

# 32. Native Desktop Philosophy

Axiom is a desktop application.

Design it accordingly.

Take advantage of the larger screen.

Use:

- sidebars

- split views

- inspectors

- keyboard navigation

- resizable panels

- contextual menus

- toolbars

- drag-and-drop where useful

- persistent study context

Do not simply design a mobile application stretched across a desktop window.

---

# 33. Key Product Principle

At every design decision, ask:

> Does this help the learner understand what to do next without limiting what they can eventually do?

That tension is fundamental to Axiom.

The interface should feel incredibly simple when used simply and extraordinarily capable when explored deeply.

---

# 34. Screens to Design

Create a coherent design system and then produce the following screens.

## Screen 1 — First Launch

Welcome to Axiom.

Create a learning workspace.

Keep this extremely simple.

---

## Screen 2 — Create Workspace

Allow the user to define:

- subject

- primary goal

- optional deadline

- optional starting level

Allow natural language.

Example:

**Subject**

Calculus II

**Goal**

"I want to deeply understand Calc II and be ready for my final in December."

---

## Screen 3 — Home

Show:

- Continue Learning

- active workspaces

- recent context

- create workspace

- optional marketplace/templates access

Do not create a dashboard full of numbers.

---

## Screen 4 — Workspace Overview

Example:

Calculus II

Primary Goal:
Master Calculus II and prepare for final exam

Continue:
Shell Method

Recommended:
Practice choosing radius and height

Quick actions:

Tutor
Practice
Visualize
Notes

Show recent concepts without clutter.

---

## Screen 5 — Active Study Session

Design an example around:

### Calculus II — Shell Method

Include:

- mathematical problem

- interactive visualization

- contextual tutor

- student's working area

- subtle session information

Make the visualization a major part of the experience.

---

## Screen 6 — Full Visualization Mode

Show an interactive solids-of-revolution visualization.

Potential controls:

- rotate

- slice

- animate revolution

- inspect washer/shell

- adjust bounds

Keep advanced controls hidden unless requested.

Make the mathematical object the hero of the screen.

---

## Screen 7 — Concept View

Example:

### Integration by Parts

Show:

- conceptual understanding

- relevant prerequisite concepts

- linked visualizations

- recent practice

- notes

- tutor access

Do not make mastery feel like a corporate KPI dashboard.

---

## Screen 8 — Workspace Modules

Show installed modules and allow the user to customize the workspace.

Example installed modules:

- Axiom Tutor

- Mathematical Visualizer

- Practice

- CAS

- Notes

- Review

Include:

Browse Modules

Keep the experience approachable to nontechnical users.

---

## Screen 9 — Module Marketplace

Create a polished marketplace for learning modules.

Show a mix of:

- official modules

- verified community modules

- workspace templates

Clearly distinguish trust level without making community modules look dangerous.

---

## Screen 10 — Module Detail

Example:

### Interactive Calculus Visualizer

Explain what educational capability it adds.

Show:

- preview

- supported concepts

- permissions/context access

- developer

- verification status

- install button

---

## Screen 11 — Goal Editing

Allow the learner to modify the workspace's primary goal.

Show that changing the goal may adjust recommendations without destroying previous work.

Example change:

From:

"Pass Calculus II."

To:

"Develop deep intuition for Calculus II before Differential Equations."

---

## Screen 12 — Command Palette

Create an advanced but elegant command palette.

Example actions:

Start practice session
Ask Tutor
Visualize concept
Open Calculus II
Review weak concepts
Create workspace
Install module

---

# 35. Design System

Create reusable components for:

- workspace cards

- concept rows

- goal displays

- module tiles

- contextual tutor panel

- mathematical canvas

- study-session toolbar

- command palette

- sidebar

- inspector

- popover

- buttons

- segmented controls

- progress indicators

- empty states

- notifications

- menus

Use a restrained component vocabulary.

Do not turn every piece of information into a card.

---

# 36. Mathematical UI

Give mathematical interaction special attention.

Equations should appear clean and typeset.

Users should eventually be able to interact directly with mathematical elements.

Think about affordances such as:

- selecting a term

- highlighting part of an expression

- asking the tutor about a specific term

- connecting an expression to its graph

- connecting a graph region to its equation

- stepping through transformations

The interface should make mathematics feel tangible.

---

# 37. Design Axiom as a System

Do not design these screens independently.

Design the underlying system so that the screens naturally emerge from it.

The same concepts, goals, modules, tutor context, visualizations, and learning state should feel connected throughout the product.

Axiom should feel like one coherent environment.

Not:

a graphing app
plus a chatbot
plus flashcards
plus notes
plus a marketplace.

Instead:

**one learning environment whose capabilities can be extended.**

---

# 38. Creative Freedom

Use this brief as a strong product foundation, not as a rigid wireframe.

You are encouraged to improve:

- information architecture

- interaction patterns

- terminology

- layout

- visual hierarchy

- navigation

when doing so clearly strengthens the product philosophy.

However, do not fundamentally alter these principles:

1. Axiom adapts to the learner.

2. Workspaces are curated learning environments.

3. Goals give the workspace purpose.

4. Concepts form the semantic learning structure.

5. Modules provide capabilities.

6. Modules are composable and extensible.

7. Visualization is a first-class learning tool.

8. The tutor understands the learner's current context.

9. Axiom is not primarily a chatbot.

10. Axiom should not depend on generic repeated questioning.

11. Complexity must be progressively disclosed.

12. The product must remain simple for ordinary users.

13. Advanced users should be able to deeply customize it.

14. Axiom should feel at home on macOS without copying Apple.

15. Simple by default. Powerful by choice.

---

# 39. Desired Result

The final product should make someone feel:

> "I can just open this and study."

Then, after using it for several weeks:

> "I didn't realize how much I could customize this."

And eventually:

> "This learning environment has become uniquely mine."

That progression is central to Axiom.

Design around it.
