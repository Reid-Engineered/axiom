# Agent Orchestration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `npm run dispatch` — a script that scans `.ai/tasks/` for ready work and
actually invokes the owning agent (Codex, Antigravity, or Claude) non-interactively, instead
of Marcus composing and pasting a resumption prompt by hand.

**Architecture:** A small set of pure, independently-testable modules under
`scripts/dispatch/` — parsing task files, checking readiness (dependencies, file conflicts,
`master`'s CI status), creating an isolated worktree and claiming the task, invoking the
right CLI, and reporting the round's outcome — wired together by one entry point run via
`tsx`. Every git/subprocess/network operation is behind an injectable function parameter so
the logic around it can be unit-tested without real git repos, real CLIs, or real network
calls, except where an integration test against a throwaway git fixture is the more honest
test (worktree creation).

**Tech Stack:** Node/TypeScript (`tsx` to run without a build step), `js-yaml` for
frontmatter parsing, Node's built-in `child_process` for git/`gh`/`codex`/`agy` invocation,
Vitest (already the repo's test runner) for tests.

**Spec:** `docs/superpowers/specs/2026-08-31-agent-orchestration-design.md`

## Global Constraints

- Branch naming: `agent/<owner>/<task-id>-<slug>` (existing convention, `owner` is one of
  `claude-code`, `codex`, `antigravity`).
- Staleness default for an `in-progress` task with no recent worklog entry: **3 days**.
- `master`'s CI must be green (latest `push`-triggered run) before **any** task in a round is
  dispatched — checked once, at the start of the round, not per task.
- Dispatch within a round is **sequential**, never parallel.
- A failed or timed-out invocation is **never retried automatically** — record it, move on.
- Worktrees live in `.worktrees/` at the repo root (git-ignored) — the fallback location
  `superpowers:using-git-worktrees` itself specifies when no other convention is declared.
- `owner: unassigned` tasks are never dispatched — reported as skipped, not inferred.
- `agy` (Antigravity CLI) is not installed on this machine yet — only the Antigravity IDE
  desktop app is present.

---

### Task 1: Task types, parser, and the `files:` frontmatter field

**Files:**
- Create: `scripts/dispatch/types.ts`
- Create: `scripts/dispatch/parseTasks.ts`
- Test: `scripts/dispatch/parseTasks.test.ts`
- Modify: `.ai/tasks/TEMPLATE.md`
- Modify: `package.json`
- Modify: `tsconfig.json` (conditionally — see Step 1)
- Modify: `.gitignore`

**Interfaces:**
- Produces: `Task` interface (`id`, `title`, `status`, `owner`, `stage`, `dependsOn: number[]`,
  `files: string[]`, `filePath: string`, `fileName: string`, `slug: string`,
  `worklogDates: string[]`), `parseTaskFile(filePath: string): Task`,
  `parseAllTasks(tasksDir: string): Task[]` — every later task in this plan consumes these.

- [ ] **Step 1: Add dependencies and confirm typecheck coverage**

```bash
npm install --save-dev js-yaml @types/js-yaml tsx
```

Read `tsconfig.json`. If its `include` array does not already cover `scripts/` (e.g. it says
`["src"]` only, or has no `include` key that would exclude it by default), add `"scripts"` to
the `include` array so `npm run typecheck` covers this new code. If `include` is absent
entirely (meaning everything under the project is already included), no change is needed —
note which case applied in your commit.

- [ ] **Step 2: Gitignore the worktree directory**

Add a line to `.gitignore`:

```
.worktrees
```

- [ ] **Step 3: Write `scripts/dispatch/types.ts`**

```typescript
export type TaskStatus = 'proposed' | 'in-progress' | 'review' | 'changes-requested' | 'done';
export type AgentOwner = 'claude-code' | 'codex' | 'antigravity' | 'unassigned';

export interface Task {
  id: number;
  title: string;
  status: TaskStatus;
  owner: AgentOwner;
  stage: string;
  dependsOn: number[];
  files: string[];
  filePath: string;
  fileName: string;
  slug: string;
  worklogDates: string[];
}
```

- [ ] **Step 4: Write the failing test for the parser**

Create `scripts/dispatch/parseTasks.test.ts`:

```typescript
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { parseAllTasks, parseTaskFile } from './parseTasks';

let dir: string;

beforeEach(() => {
  dir = mkdtempSync(join(tmpdir(), 'dispatch-parse-'));
});

afterEach(() => {
  rmSync(dir, { recursive: true, force: true });
});

const TASK_A = `---
id: 1
title: Example task
status: proposed
owner: codex
stage: 1
depends_on: [2]
files: [src/foo.ts, src/foo.test.ts]
---

## Scope

Does a thing.

## Worklog

- 2026-08-20 — started, claimed by codex
- 2026-08-22 — made progress
`;

const TASK_B = `---
id: 2
title: Dependency task
status: done
owner: claude-code
stage: 1
depends_on: []
files: []
---

## Worklog

- 2026-08-15 — done
`;

describe('parseTaskFile', () => {
  it('parses frontmatter, files, and worklog dates', () => {
    const path = join(dir, '001-example-task.md');
    writeFileSync(path, TASK_A);

    const task = parseTaskFile(path);

    expect(task.id).toBe(1);
    expect(task.title).toBe('Example task');
    expect(task.status).toBe('proposed');
    expect(task.owner).toBe('codex');
    expect(task.dependsOn).toEqual([2]);
    expect(task.files).toEqual(['src/foo.ts', 'src/foo.test.ts']);
    expect(task.fileName).toBe('001-example-task.md');
    expect(task.slug).toBe('example-task');
    expect(task.worklogDates).toEqual(['2026-08-20', '2026-08-22']);
  });

  it('defaults owner to unassigned and files/dependsOn to empty when absent', () => {
    const path = join(dir, '003-minimal.md');
    writeFileSync(
      path,
      '---\nid: 3\ntitle: Minimal\nstatus: proposed\nstage: 1\n---\n\n## Worklog\n',
    );

    const task = parseTaskFile(path);

    expect(task.owner).toBe('unassigned');
    expect(task.files).toEqual([]);
    expect(task.dependsOn).toEqual([]);
    expect(task.worklogDates).toEqual([]);
  });
});

describe('parseAllTasks', () => {
  it('parses every task file in the directory and skips TEMPLATE.md', () => {
    writeFileSync(join(dir, '001-example-task.md'), TASK_A);
    writeFileSync(join(dir, '002-dependency-task.md'), TASK_B);
    writeFileSync(join(dir, 'TEMPLATE.md'), '---\nid: 0\ntitle: t\nstatus: proposed\n---\n');

    const tasks = parseAllTasks(dir);

    expect(tasks).toHaveLength(2);
    expect(tasks.map((t) => t.id).sort()).toEqual([1, 2]);
  });
});
```

- [ ] **Step 5: Run the test to verify it fails**

Run: `npx vitest run scripts/dispatch/parseTasks.test.ts`
Expected: FAIL — `parseTasks` module not found.

- [ ] **Step 6: Write `scripts/dispatch/parseTasks.ts`**

```typescript
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import yaml from 'js-yaml';
import type { AgentOwner, Task, TaskStatus } from './types';

const FRONTMATTER_RE = /^---\n([\s\S]*?)\n---\n?([\s\S]*)$/;
const WORKLOG_DATE_RE = /^-\s+(\d{4}-\d{2}-\d{2})\s/gm;
const SLUG_FROM_FILENAME_RE = /^\d+-(.+)\.md$/;

export function parseTaskFile(filePath: string): Task {
  const raw = readFileSync(filePath, 'utf8');
  const match = raw.match(FRONTMATTER_RE);
  if (!match) {
    throw new Error(`${filePath}: no YAML frontmatter found`);
  }
  const [, frontmatterRaw, body] = match;
  const fm = (yaml.load(frontmatterRaw) ?? {}) as Record<string, unknown>;

  const fileName = filePath.split('/').pop() ?? filePath;
  const slugMatch = fileName.match(SLUG_FROM_FILENAME_RE);
  const slug = slugMatch ? slugMatch[1] : fileName.replace(/\.md$/, '');

  const worklogSection = body.split(/^## Worklog\s*$/m)[1]?.split(/^## /m)[0] ?? '';
  const worklogDates = Array.from(worklogSection.matchAll(WORKLOG_DATE_RE))
    .map((m) => m[1])
    .sort();

  return {
    id: Number(fm.id),
    title: String(fm.title ?? ''),
    status: fm.status as TaskStatus,
    owner: (fm.owner as AgentOwner) ?? 'unassigned',
    stage: String(fm.stage ?? ''),
    dependsOn: Array.isArray(fm.depends_on) ? (fm.depends_on as unknown[]).map(Number) : [],
    files: Array.isArray(fm.files) ? (fm.files as unknown[]).map(String) : [],
    filePath,
    fileName,
    slug,
    worklogDates,
  };
}

export function parseAllTasks(tasksDir: string): Task[] {
  const entries = readdirSync(tasksDir, { withFileTypes: true });
  const files = entries
    .filter((e) => e.isFile() && e.name.endsWith('.md') && e.name !== 'TEMPLATE.md')
    .map((e) => join(tasksDir, e.name));
  return files.map(parseTaskFile);
}
```

- [ ] **Step 7: Run the test to verify it passes**

Run: `npx vitest run scripts/dispatch/parseTasks.test.ts`
Expected: PASS, 3 tests.

- [ ] **Step 8: Add the `files:` field to `TEMPLATE.md`**

Find this text in `.ai/tasks/TEMPLATE.md`:

```
---
id: 000
title: <short title>
status: proposed
owner: <unassigned | agent name>
stage: <ROADMAP.md stage number>
depends_on: []
---
```

Replace with:

```
---
id: 000
title: <short title>
status: proposed
owner: <unassigned | agent name>
stage: <ROADMAP.md stage number>
depends_on: []
files: []
---
```

Then find the `## Plan` section's body text:

```
## Plan

Files to be created or touched. If this list grows materially once work starts, that's a
signal the task is bigger than scoped — see `.ai/lifecycle.md` on splitting.
```

Replace with:

```
## Plan

Files to be created or touched — also listed in the frontmatter's `files:` field, which the
dispatch script (`scripts/dispatch/`) uses to detect conflicts between tasks; keep the two in
sync. If this list grows materially once work starts, that's a signal the task is bigger than
scoped — see `.ai/lifecycle.md` on splitting.
```

- [ ] **Step 9: Run the full test suite and typecheck**

Run: `npx vitest run && npm run typecheck`
Expected: all pass, zero typecheck errors.

- [ ] **Step 10: Commit**

```bash
git add scripts/dispatch/types.ts scripts/dispatch/parseTasks.ts scripts/dispatch/parseTasks.test.ts .ai/tasks/TEMPLATE.md package.json package-lock.json tsconfig.json .gitignore
git commit -m "feat(dispatch): add task types, frontmatter parser, files: field"
```

---

### Task 2: Readiness checks

**Files:**
- Create: `scripts/dispatch/readiness.ts`
- Test: `scripts/dispatch/readiness.test.ts`

**Interfaces:**
- Consumes: `Task` from `./types` (Task 1).
- Produces: `checkReadiness(task: Task, tasksById: Map<number, Task>, inProgressTasks: Task[], referenceDate: Date): ReadinessResult` where `ReadinessResult = { ready: boolean; reasons: string[] }` — Task 6 (round orchestrator) calls this directly.

- [ ] **Step 1: Write the failing tests**

Create `scripts/dispatch/readiness.test.ts`:

```typescript
import { describe, expect, it } from 'vitest';
import { checkReadiness } from './readiness';
import type { Task } from './types';

function makeTask(overrides: Partial<Task> = {}): Task {
  return {
    id: 1,
    title: 'Task',
    status: 'proposed',
    owner: 'codex',
    stage: '1',
    dependsOn: [],
    files: [],
    filePath: '/tmp/001-task.md',
    fileName: '001-task.md',
    slug: 'task',
    worklogDates: [],
    ...overrides,
  };
}

const NOW = new Date('2026-08-31T00:00:00Z');

describe('checkReadiness', () => {
  it('is ready for a fresh proposed task with no dependencies or conflicts', () => {
    const task = makeTask();
    const result = checkReadiness(task, new Map([[1, task]]), [], NOW);
    expect(result).toEqual({ ready: true, reasons: [] });
  });

  it('is not ready when owner is unassigned', () => {
    const task = makeTask({ owner: 'unassigned' });
    const result = checkReadiness(task, new Map([[1, task]]), [], NOW);
    expect(result.ready).toBe(false);
    expect(result.reasons).toContain('owner is unassigned');
  });

  it('is not ready when a dependency is not done', () => {
    const dep = makeTask({ id: 2, status: 'in-progress' });
    const task = makeTask({ dependsOn: [2] });
    const result = checkReadiness(
      task,
      new Map([[1, task], [2, dep]]),
      [],
      NOW,
    );
    expect(result.ready).toBe(false);
    expect(result.reasons).toContain('unsatisfied dependencies');
  });

  it('is ready when every dependency is done', () => {
    const dep = makeTask({ id: 2, status: 'done' });
    const task = makeTask({ dependsOn: [2] });
    const result = checkReadiness(
      task,
      new Map([[1, task], [2, dep]]),
      [],
      NOW,
    );
    expect(result.ready).toBe(true);
  });

  it('is not ready when files overlap an in-progress task', () => {
    const other = makeTask({ id: 2, status: 'in-progress', files: ['src/foo.ts'] });
    const task = makeTask({ files: ['src/foo.ts', 'src/bar.ts'] });
    const result = checkReadiness(task, new Map([[1, task], [2, other]]), [other], NOW);
    expect(result.ready).toBe(false);
    expect(result.reasons).toContain('file conflict with another in-progress task');
  });

  it('an in-progress task with a worklog entry within 3 days is not a candidate', () => {
    const task = makeTask({ status: 'in-progress', worklogDates: ['2026-08-30'] });
    const result = checkReadiness(task, new Map([[1, task]]), [task], NOW);
    expect(result.ready).toBe(false);
    expect(result.reasons).toContain('not a candidate (status is not proposed or stale in-progress)');
  });

  it('an in-progress task stale for more than 3 days is a candidate', () => {
    const task = makeTask({ status: 'in-progress', worklogDates: ['2026-08-20'] });
    const result = checkReadiness(task, new Map([[1, task]]), [task], NOW);
    expect(result.ready).toBe(true);
  });

  it('a done task is never a candidate', () => {
    const task = makeTask({ status: 'done' });
    const result = checkReadiness(task, new Map([[1, task]]), [], NOW);
    expect(result.ready).toBe(false);
  });
});
```

- [ ] **Step 2: Run to verify it fails**

Run: `npx vitest run scripts/dispatch/readiness.test.ts`
Expected: FAIL — module not found.

- [ ] **Step 3: Write `scripts/dispatch/readiness.ts`**

```typescript
import type { Task } from './types';

const STALE_DAYS = 3;
const MS_PER_DAY = 24 * 60 * 60 * 1000;

export function isStale(task: Task, referenceDate: Date, staleDays = STALE_DAYS): boolean {
  if (task.status !== 'in-progress') return false;
  if (task.worklogDates.length === 0) return true;
  const last = task.worklogDates[task.worklogDates.length - 1];
  const lastDate = new Date(`${last}T00:00:00Z`);
  return referenceDate.getTime() - lastDate.getTime() > staleDays * MS_PER_DAY;
}

export function isCandidate(task: Task, referenceDate: Date): boolean {
  if (task.status === 'proposed') return true;
  if (task.status === 'in-progress') return isStale(task, referenceDate);
  return false;
}

export function dependenciesSatisfied(task: Task, tasksById: Map<number, Task>): boolean {
  return task.dependsOn.every((depId) => tasksById.get(depId)?.status === 'done');
}

export function filesOverlap(a: string[], b: string[]): boolean {
  const setB = new Set(b);
  return a.some((f) => setB.has(f));
}

export function hasFileConflict(task: Task, inProgressTasks: Task[]): boolean {
  return inProgressTasks
    .filter((t) => t.id !== task.id)
    .some((t) => filesOverlap(task.files, t.files));
}

export interface ReadinessResult {
  ready: boolean;
  reasons: string[];
}

export function checkReadiness(
  task: Task,
  tasksById: Map<number, Task>,
  inProgressTasks: Task[],
  referenceDate: Date,
): ReadinessResult {
  const reasons: string[] = [];
  if (!isCandidate(task, referenceDate)) {
    reasons.push('not a candidate (status is not proposed or stale in-progress)');
  }
  if (task.owner === 'unassigned') {
    reasons.push('owner is unassigned');
  }
  if (!dependenciesSatisfied(task, tasksById)) {
    reasons.push('unsatisfied dependencies');
  }
  if (hasFileConflict(task, inProgressTasks)) {
    reasons.push('file conflict with another in-progress task');
  }
  return { ready: reasons.length === 0, reasons };
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `npx vitest run scripts/dispatch/readiness.test.ts`
Expected: PASS, 8 tests.

- [ ] **Step 5: Commit**

```bash
git add scripts/dispatch/readiness.ts scripts/dispatch/readiness.test.ts
git commit -m "feat(dispatch): add readiness checks (deps, file conflicts, staleness)"
```

---

### Task 3: `master` CI status check

**Files:**
- Create: `scripts/dispatch/ciStatus.ts`
- Test: `scripts/dispatch/ciStatus.test.ts`

**Interfaces:**
- Produces: `getMasterCiStatus(repoSlug: string, runGhApi?: GhApiRunner): Promise<CiConclusion>` where `CiConclusion = 'success' | 'failure' | 'pending' | 'unknown'` and `GhApiRunner = (args: string[]) => Promise<string>` — Task 6 calls this directly, with the real `gh` runner (no override) in production.

- [ ] **Step 1: Write the failing tests**

Create `scripts/dispatch/ciStatus.test.ts`:

```typescript
import { describe, expect, it } from 'vitest';
import { getMasterCiStatus } from './ciStatus';

describe('getMasterCiStatus', () => {
  it('returns success when the latest push run succeeded', async () => {
    const status = await getMasterCiStatus('owner/repo', async () =>
      JSON.stringify([{ conclusion: 'success', status: 'completed' }]),
    );
    expect(status).toBe('success');
  });

  it('returns failure when the latest push run failed', async () => {
    const status = await getMasterCiStatus('owner/repo', async () =>
      JSON.stringify([{ conclusion: 'failure', status: 'completed' }]),
    );
    expect(status).toBe('failure');
  });

  it('returns pending when the run has not completed', async () => {
    const status = await getMasterCiStatus('owner/repo', async () =>
      JSON.stringify([{ conclusion: '', status: 'in_progress' }]),
    );
    expect(status).toBe('pending');
  });

  it('returns unknown when there are no runs', async () => {
    const status = await getMasterCiStatus('owner/repo', async () => JSON.stringify([]));
    expect(status).toBe('unknown');
  });

  it('calls gh with the expected arguments', async () => {
    const calls: string[][] = [];
    await getMasterCiStatus('owner/repo', async (args) => {
      calls.push(args);
      return JSON.stringify([{ conclusion: 'success', status: 'completed' }]);
    });
    expect(calls).toEqual([
      [
        'run', 'list',
        '--repo', 'owner/repo',
        '--branch', 'master',
        '--event', 'push',
        '--limit', '1',
        '--json', 'conclusion,status',
      ],
    ]);
  });
});
```

- [ ] **Step 2: Run to verify it fails**

Run: `npx vitest run scripts/dispatch/ciStatus.test.ts`
Expected: FAIL — module not found.

- [ ] **Step 3: Write `scripts/dispatch/ciStatus.ts`**

```typescript
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);

export type CiConclusion = 'success' | 'failure' | 'pending' | 'unknown';
export type GhApiRunner = (args: string[]) => Promise<string>;

export const realGhApiRunner: GhApiRunner = async (args) => {
  const { stdout } = await execFileAsync('gh', args);
  return stdout;
};

export async function getMasterCiStatus(
  repoSlug: string,
  runGhApi: GhApiRunner = realGhApiRunner,
): Promise<CiConclusion> {
  const stdout = await runGhApi([
    'run', 'list',
    '--repo', repoSlug,
    '--branch', 'master',
    '--event', 'push',
    '--limit', '1',
    '--json', 'conclusion,status',
  ]);
  const runs = JSON.parse(stdout) as Array<{ conclusion: string; status: string }>;
  if (runs.length === 0) return 'unknown';
  const [run] = runs;
  if (run.status !== 'completed') return 'pending';
  if (run.conclusion === 'success') return 'success';
  if (run.conclusion === 'failure') return 'failure';
  return 'unknown';
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `npx vitest run scripts/dispatch/ciStatus.test.ts`
Expected: PASS, 5 tests.

- [ ] **Step 5: Commit**

```bash
git add scripts/dispatch/ciStatus.ts scripts/dispatch/ciStatus.test.ts
git commit -m "feat(dispatch): add master CI status check via gh"
```

---

### Task 4: Worktree creation and task claiming

**Files:**
- Create: `scripts/dispatch/worktree.ts`
- Test: `scripts/dispatch/worktree.test.ts`

**Interfaces:**
- Consumes: `Task` from `./types` (Task 1) — needs `id`, `owner`, `slug`, `fileName`.
- Produces: `createWorktreeAndClaim(task: Task, repoRoot: string, worktreeBaseDir: string, today: string): ClaimResult` where `ClaimResult = { worktreePath: string; branch: string }` — Task 6 calls this.

- [ ] **Step 1: Write the failing test**

Create `scripts/dispatch/worktree.test.ts`. This is an integration test against a real,
throwaway git repository (worktree creation is real git state — mocking it would test
nothing):

```typescript
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { createWorktreeAndClaim } from './worktree';
import type { Task } from './types';

let repoRoot: string;
let worktreeBaseDir: string;

beforeEach(() => {
  repoRoot = mkdtempSync(join(tmpdir(), 'dispatch-repo-'));
  execFileSync('git', ['init', '-b', 'master'], { cwd: repoRoot });
  execFileSync('git', ['config', 'user.email', 'test@example.com'], { cwd: repoRoot });
  execFileSync('git', ['config', 'user.name', 'Test'], { cwd: repoRoot });

  mkdirSync(join(repoRoot, '.ai', 'tasks'), { recursive: true });
  writeFileSync(
    join(repoRoot, '.ai', 'tasks', '001-example-task.md'),
    '---\nid: 1\ntitle: Example task\nstatus: proposed\nowner: codex\nstage: 1\ndepends_on: []\nfiles: []\n---\n\n## Worklog\n',
  );
  execFileSync('git', ['add', '.'], { cwd: repoRoot });
  execFileSync('git', ['commit', '-m', 'initial'], { cwd: repoRoot });

  worktreeBaseDir = mkdtempSync(join(tmpdir(), 'dispatch-worktrees-'));
});

afterEach(() => {
  rmSync(worktreeBaseDir, { recursive: true, force: true });
  rmSync(repoRoot, { recursive: true, force: true });
});

function makeTask(overrides: Partial<Task> = {}): Task {
  return {
    id: 1,
    title: 'Example task',
    status: 'proposed',
    owner: 'codex',
    stage: '1',
    dependsOn: [],
    files: [],
    filePath: join(repoRoot, '.ai', 'tasks', '001-example-task.md'),
    fileName: '001-example-task.md',
    slug: 'example-task',
    worklogDates: [],
    ...overrides,
  };
}

describe('createWorktreeAndClaim', () => {
  it('creates a worktree, branch, and claim commit', () => {
    const task = makeTask();

    const { worktreePath, branch } = createWorktreeAndClaim(
      task,
      repoRoot,
      worktreeBaseDir,
      '2026-08-31',
    );

    expect(branch).toBe('agent/codex/1-example-task');

    const branches = execFileSync('git', ['branch', '--list', branch], { cwd: repoRoot })
      .toString()
      .trim();
    expect(branches).toContain(branch);

    const claimedContent = readFileSync(join(worktreePath, '.ai/tasks/001-example-task.md'), 'utf8');
    expect(claimedContent).toContain('status: in-progress');
    expect(claimedContent).toContain('- 2026-08-31 — dispatched to codex by dispatch round');

    const log = execFileSync('git', ['log', '--oneline', '-1'], { cwd: worktreePath })
      .toString()
      .trim();
    expect(log).toContain('claim for codex, dispatch round');
  });
});
```

- [ ] **Step 2: Run to verify it fails**

Run: `npx vitest run scripts/dispatch/worktree.test.ts`
Expected: FAIL — module not found.

- [ ] **Step 3: Write `scripts/dispatch/worktree.ts`**

```typescript
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Task } from './types';

export interface ClaimResult {
  worktreePath: string;
  branch: string;
}

export function createWorktreeAndClaim(
  task: Task,
  repoRoot: string,
  worktreeBaseDir: string,
  today: string,
): ClaimResult {
  const branch = `agent/${task.owner}/${task.id}-${task.slug}`;
  const worktreePath = join(worktreeBaseDir, `${task.id}-${task.slug}`);

  execFileSync('git', ['worktree', 'add', worktreePath, '-b', branch], { cwd: repoRoot });

  const taskFileInWorktree = join(worktreePath, '.ai', 'tasks', task.fileName);
  const raw = readFileSync(taskFileInWorktree, 'utf8');
  const claimed = raw
    .replace(/^status:\s*proposed\s*$/m, 'status: in-progress')
    .replace(
      /^## Worklog\s*$/m,
      `## Worklog\n\n- ${today} — dispatched to ${task.owner} by dispatch round`,
    );
  writeFileSync(taskFileInWorktree, claimed);

  execFileSync('git', ['add', taskFileInWorktree], { cwd: worktreePath });
  execFileSync(
    'git',
    ['commit', '-m', `chore(${task.id}): claim for ${task.owner}, dispatch round`],
    { cwd: worktreePath },
  );

  return { worktreePath, branch };
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `npx vitest run scripts/dispatch/worktree.test.ts`
Expected: PASS, 1 test.

- [ ] **Step 5: Commit**

```bash
git add scripts/dispatch/worktree.ts scripts/dispatch/worktree.test.ts
git commit -m "feat(dispatch): add worktree creation and task-claim commit"
```

---

### Task 5: Per-agent invocation

**Files:**
- Create: `scripts/dispatch/invoke.ts`
- Test: `scripts/dispatch/invoke.test.ts`

**Interfaces:**
- Consumes: `Task` from `./types` (Task 1).
- Produces: `invokeAgent(task: Task, worktreePath: string, execer?: Execer): Promise<InvocationResult>` where `InvocationResult = { status: 'done' | 'errored' | 'needs-manual-dispatch'; output: string }` and `Execer = (command: string, args: string[], options: { cwd: string; timeout: number }) => Promise<{ stdout: string; stderr: string }>` — Task 6 calls this.

- [ ] **Step 1: Write the failing tests**

Create `scripts/dispatch/invoke.test.ts`:

```typescript
import { describe, expect, it } from 'vitest';
import { invokeAgent } from './invoke';
import type { Task } from './types';

function makeTask(overrides: Partial<Task> = {}): Task {
  return {
    id: 1,
    title: 'Task',
    status: 'proposed',
    owner: 'codex',
    stage: '1',
    dependsOn: [],
    files: [],
    filePath: '/tmp/001-task.md',
    fileName: '001-task.md',
    slug: 'task',
    worklogDates: [],
    ...overrides,
  };
}

describe('invokeAgent', () => {
  it('returns needs-manual-dispatch for claude-code without calling execer', async () => {
    const task = makeTask({ owner: 'claude-code' });
    let called = false;
    const result = await invokeAgent(task, '/tmp/worktree', async () => {
      called = true;
      return { stdout: '', stderr: '' };
    });
    expect(called).toBe(false);
    expect(result.status).toBe('needs-manual-dispatch');
    expect(result.output).toContain('/tmp/worktree');
  });

  it('invokes codex exec with the dispatch prompt in the worktree', async () => {
    const task = makeTask({ owner: 'codex' });
    const calls: Array<{ command: string; args: string[]; cwd: string }> = [];
    const result = await invokeAgent(task, '/tmp/worktree', async (command, args, options) => {
      calls.push({ command, args, cwd: options.cwd });
      return { stdout: 'codex output', stderr: '' };
    });
    expect(calls).toHaveLength(1);
    expect(calls[0].command).toBe('codex');
    expect(calls[0].args[0]).toBe('exec');
    expect(calls[0].cwd).toBe('/tmp/worktree');
    expect(result.status).toBe('done');
    expect(result.output).toBe('codex output');
  });

  it('invokes agy with the correct headless flags in the worktree', async () => {
    const task = makeTask({ owner: 'antigravity' });
    const calls: Array<{ command: string; args: string[]; cwd: string }> = [];
    await invokeAgent(task, '/tmp/worktree', async (command, args, options) => {
      calls.push({ command, args, cwd: options.cwd });
      return { stdout: '{}', stderr: '' };
    });
    expect(calls[0].command).toBe('agy');
    expect(calls[0].args).toContain('-p');
    expect(calls[0].args).toContain('--output-format');
    expect(calls[0].args).toContain('json');
    expect(calls[0].args).toContain('--dangerously-skip-permissions');
    expect(calls[0].args).toContain('--print-timeout');
    expect(calls[0].cwd).toBe('/tmp/worktree');
  });

  it('returns errored when execer throws', async () => {
    const task = makeTask({ owner: 'codex' });
    const result = await invokeAgent(task, '/tmp/worktree', async () => {
      throw new Error('boom');
    });
    expect(result.status).toBe('errored');
    expect(result.output).toContain('boom');
  });
});
```

- [ ] **Step 2: Run to verify it fails**

Run: `npx vitest run scripts/dispatch/invoke.test.ts`
Expected: FAIL — module not found.

- [ ] **Step 3: Write `scripts/dispatch/invoke.ts`**

```typescript
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import type { Task } from './types';

const execFileAsync = promisify(execFile);

export interface InvocationResult {
  status: 'done' | 'errored' | 'needs-manual-dispatch';
  output: string;
}

export type Execer = (
  command: string,
  args: string[],
  options: { cwd: string; timeout: number },
) => Promise<{ stdout: string; stderr: string }>;

export const realExecer: Execer = async (command, args, options) => {
  const result = await execFileAsync(command, args, options);
  return { stdout: result.stdout, stderr: result.stderr };
};

const DISPATCH_PROMPT =
  'Read the task file for this branch in .ai/tasks/ in full, then AGENTS.md and ' +
  'CLAUDE.md (or the equivalent doc for your tool), and pick up this task following ' +
  'the workflow those docs describe.';

const TIMEOUT_MS = 30 * 60 * 1000;

export async function invokeAgent(
  task: Task,
  worktreePath: string,
  execer: Execer = realExecer,
): Promise<InvocationResult> {
  if (task.owner === 'claude-code') {
    return {
      status: 'needs-manual-dispatch',
      output: `Ready for Claude to pick up via the Agent tool at ${worktreePath}.`,
    };
  }

  try {
    if (task.owner === 'codex') {
      const { stdout } = await execer('codex', ['exec', DISPATCH_PROMPT], {
        cwd: worktreePath,
        timeout: TIMEOUT_MS,
      });
      return { status: 'done', output: stdout };
    }

    if (task.owner === 'antigravity') {
      const { stdout } = await execer(
        'agy',
        [
          '-p', DISPATCH_PROMPT,
          '--output-format', 'json',
          '--dangerously-skip-permissions',
          '--print-timeout', '30m',
        ],
        { cwd: worktreePath, timeout: TIMEOUT_MS },
      );
      return { status: 'done', output: stdout };
    }

    return { status: 'errored', output: `Unknown owner: ${task.owner}` };
  } catch (err) {
    return { status: 'errored', output: String(err) };
  }
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `npx vitest run scripts/dispatch/invoke.test.ts`
Expected: PASS, 4 tests.

- [ ] **Step 5: Commit**

```bash
git add scripts/dispatch/invoke.ts scripts/dispatch/invoke.test.ts
git commit -m "feat(dispatch): add per-agent invocation (codex, antigravity, claude-code)"
```

---

### Task 6: Report formatting, round orchestrator, and `npm run dispatch`

**Files:**
- Create: `scripts/dispatch/report.ts`
- Create: `scripts/dispatch/index.ts`
- Test: `scripts/dispatch/report.test.ts`
- Modify: `package.json`

**Interfaces:**
- Consumes: everything from Tasks 1-5 (`Task`, `checkReadiness`, `getMasterCiStatus`,
  `createWorktreeAndClaim`, `invokeAgent`).
- Produces: `formatReport(entries: RoundEntry[], skipped: SkippedEntry[]): string` where
  `RoundEntry = { task: Task; worktreePath: string; branch: string; result: InvocationResult }`
  and `SkippedEntry = { task: Task; reasons: string[] }`; and the `index.ts` entry point run
  via `npm run dispatch`.

- [ ] **Step 1: Write the failing test for the report**

Create `scripts/dispatch/report.test.ts`:

```typescript
import { describe, expect, it } from 'vitest';
import { formatReport } from './report';
import type { Task } from './types';

function makeTask(overrides: Partial<Task> = {}): Task {
  return {
    id: 1,
    title: 'Example task',
    status: 'proposed',
    owner: 'codex',
    stage: '1',
    dependsOn: [],
    files: [],
    filePath: '/tmp/001-task.md',
    fileName: '001-task.md',
    slug: 'example-task',
    worklogDates: [],
    ...overrides,
  };
}

describe('formatReport', () => {
  it('reports dispatched and skipped tasks', () => {
    const task = makeTask();
    const report = formatReport(
      [
        {
          task,
          worktreePath: '/tmp/worktrees/1-example-task',
          branch: 'agent/codex/1-example-task',
          result: { status: 'done', output: 'ok' },
        },
      ],
      [
        {
          task: makeTask({ id: 2, title: 'Blocked task', owner: 'unassigned' }),
          reasons: ['owner is unassigned'],
        },
      ],
    );

    expect(report).toContain('1 dispatched, 1 skipped');
    expect(report).toContain('[done] #1 Example task -> codex');
    expect(report).toContain('agent/codex/1-example-task');
    expect(report).toContain('#2 Blocked task: owner is unassigned');
  });
});
```

- [ ] **Step 2: Run to verify it fails**

Run: `npx vitest run scripts/dispatch/report.test.ts`
Expected: FAIL — module not found.

- [ ] **Step 3: Write `scripts/dispatch/report.ts`**

```typescript
import type { InvocationResult } from './invoke';
import type { Task } from './types';

export interface RoundEntry {
  task: Task;
  worktreePath: string;
  branch: string;
  result: InvocationResult;
}

export interface SkippedEntry {
  task: Task;
  reasons: string[];
}

export function formatReport(entries: RoundEntry[], skipped: SkippedEntry[]): string {
  const lines: string[] = [];
  lines.push(`Dispatch round: ${entries.length} dispatched, ${skipped.length} skipped`);
  lines.push('');
  for (const e of entries) {
    lines.push(`  [${e.result.status}] #${e.task.id} ${e.task.title} -> ${e.task.owner}`);
    lines.push(`    branch: ${e.branch}`);
    lines.push(`    worktree: ${e.worktreePath}`);
  }
  if (skipped.length > 0) {
    lines.push('');
    lines.push('Skipped:');
    for (const s of skipped) {
      lines.push(`  #${s.task.id} ${s.task.title}: ${s.reasons.join(', ')}`);
    }
  }
  return lines.join('\n');
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `npx vitest run scripts/dispatch/report.test.ts`
Expected: PASS, 1 test.

- [ ] **Step 5: Write `scripts/dispatch/index.ts`**

```typescript
#!/usr/bin/env node
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
import { parseAllTasks } from './parseTasks';
import { checkReadiness } from './readiness';
import { getMasterCiStatus } from './ciStatus';
import { createWorktreeAndClaim } from './worktree';
import { invokeAgent } from './invoke';
import { formatReport, type RoundEntry, type SkippedEntry } from './report';
import type { Task } from './types';

async function main(): Promise<void> {
  const scriptDir = dirname(fileURLToPath(import.meta.url));
  const repoRoot = resolve(scriptDir, '..', '..');
  const tasksDir = resolve(repoRoot, '.ai', 'tasks');
  const worktreeBaseDir = resolve(repoRoot, '.worktrees');

  const tasks = parseAllTasks(tasksDir);
  const tasksById = new Map(tasks.map((t) => [t.id, t]));
  const inProgressTasks = tasks.filter((t) => t.status === 'in-progress');

  const ciStatus = await getMasterCiStatus('Reid-Engineered/axiom');
  if (ciStatus !== 'success') {
    console.log(`master's CI is not green (${ciStatus}) -- stopping without dispatching.`);
    return;
  }

  const referenceDate = new Date();
  const todayStr = referenceDate.toISOString().slice(0, 10);

  const entries: RoundEntry[] = [];
  const skipped: SkippedEntry[] = [];

  for (const task of tasks) {
    const readiness = checkReadiness(task, tasksById, inProgressTasks, referenceDate);
    if (!readiness.ready) {
      skipped.push({ task, reasons: readiness.reasons });
      continue;
    }

    try {
      const { worktreePath, branch } = createWorktreeAndClaim(
        task,
        repoRoot,
        worktreeBaseDir,
        todayStr,
      );
      const result = await invokeAgent(task, worktreePath);
      entries.push({ task, worktreePath, branch, result });
    } catch (err) {
      entries.push({
        task,
        worktreePath: '(not created)',
        branch: '(not created)',
        result: { status: 'errored', output: String(err) },
      });
    }
  }

  console.log(formatReport(entries, skipped));
}

main().catch((err) => {
  console.error(err);
  process.exitCode = 1;
});
```

- [ ] **Step 6: Wire up `npm run dispatch`**

In `package.json`'s `"scripts"` block, add:

```json
"dispatch": "tsx scripts/dispatch/index.ts"
```

- [ ] **Step 7: Run the full suite and typecheck**

Run: `npx vitest run && npm run typecheck`
Expected: all pass, zero typecheck errors.

- [ ] **Step 8: Commit**

```bash
git add scripts/dispatch/report.ts scripts/dispatch/report.test.ts scripts/dispatch/index.ts package.json
git commit -m "feat(dispatch): add report formatting, round orchestrator, npm run dispatch"
```

---

### Task 7: End-to-end validation against the live repo

**Files:** none new — this task validates Tasks 1-6 against the real `axiom` repo, the same
way sub-project 1's CI was validated with a real PR rather than trusting a read-through.

- [ ] **Step 1: Install and authenticate `agy`**

Check whether `agy` is already on `PATH`:

```bash
type agy
```

If not found, follow Google's install instructions
(https://antigravity.google/docs/cli/overview/) to install it, then authenticate once
interactively (`agy` with no flags, complete whatever sign-in flow it presents) so headless
mode can use cached credentials afterward, per the docs: "Headless mode uses your cached
credentials. Authenticate once with an interactive `agy` session first."

If installation hits a blocker specific to this environment (mirroring the `gh`/sudo friction
from sub-project 1), work around it the same way: find a no-sudo install path, or if none
exists, note the blocker plainly and proceed with Codex-only validation below — Antigravity
validation becomes a follow-up rather than blocking this task.

- [ ] **Step 2: Confirm `master`'s CI is green**

```bash
gh run list --repo Reid-Engineered/axiom --branch master --event push --limit 1 --json conclusion,status
```

Expected: `"conclusion":"success"`. If not, resolve that first (the dispatcher itself will
refuse to run otherwise).

- [ ] **Step 3: Create one trivial validation task**

Create `.ai/tasks/054-dispatch-smoke-test.md` (check `.ai/tasks/` and `.ai/tasks/_archive/`
first to confirm 054 is actually the next free id — use whatever id is actually free if not):

```markdown
---
id: 054
title: Dispatch smoke test
status: proposed
owner: codex
stage: N/A — dispatcher validation, not a ROADMAP.md stage
depends_on: []
files: [docs/dispatch-smoke-test.md]
---

## Scope

Validation-only task for `scripts/dispatch/`. Create a file `docs/dispatch-smoke-test.md`
containing exactly the line `dispatch smoke test ok` (nothing else). This task exists only to
prove the dispatch script actually invokes an agent end-to-end; it is not real product work
and will be deleted (along with its branch) once it's served that purpose.

## Plan

Files to be created or touched:
- Create: `docs/dispatch-smoke-test.md`

## Worklog

- 2026-08-31 — created for dispatch validation
```

(replace `2026-08-31` above with the actual date you're running this step on, in `YYYY-MM-DD` format, before writing the file)

Commit it directly to `master` (this is validation scaffolding, not a real task — same
treatment as sub-project 1's own throwaway validation commits):

```bash
git add .ai/tasks/054-dispatch-smoke-test.md
git commit -m "chore(dispatch): add smoke-test task for dispatcher validation"
git push origin master
```

- [ ] **Step 4: Run a dispatch round**

```bash
npm run dispatch
```

Expected: the report shows task 054 dispatched to `codex`, with a real worktree path and
`[done]` status (or `[errored]` with a specific, readable reason if Codex genuinely
couldn't complete it — either way, confirm the worktree and branch were created and the
claim commit landed).

- [ ] **Step 5: Verify the claim and the agent's work**

```bash
cat .worktrees/054-dispatch-smoke-test/.ai/tasks/054-dispatch-smoke-test.md
cat .worktrees/054-dispatch-smoke-test/docs/dispatch-smoke-test.md
```

Expected: the task file shows `status: in-progress` and a dispatch worklog line; the smoke
test file exists with the expected content (proving Codex actually read the task file and
acted on it, not just that a process launched).

- [ ] **Step 6: If `agy` was successfully installed in Step 1, repeat with an Antigravity-owned task**

Create a second trivial task the same shape as Step 3 but with `owner: antigravity` and a
different target file (e.g. `docs/dispatch-smoke-test-antigravity.md`), commit it to
`master`, run `npm run dispatch` again, and confirm the same way. If `agy` wasn't installed,
skip this step and record it as a follow-up instead.

- [ ] **Step 7: Validate the file-conflict skip**

Create two more trivial tasks with **overlapping** `files:` entries (both listing
`docs/dispatch-conflict-test.md`), one `status: proposed`, the other `status: in-progress`
with a recent worklog date (so it counts as an active conflict, not a stale one). Commit both
to `master`, run `npm run dispatch`, and confirm the report shows the `proposed` one in the
**skipped** section with reason `file conflict with another in-progress task` — not
dispatched.

- [ ] **Step 8: Clean up all validation artifacts**

```bash
git worktree remove .worktrees/054-dispatch-smoke-test --force
```

(repeat for any other worktrees created in Steps 6-7)

```bash
git branch -D agent/codex/054-dispatch-smoke-test
```

(repeat for other branches created)

On `master`, delete the smoke-test task files and any files an agent actually created during
validation, then commit:

```bash
git rm .ai/tasks/054-dispatch-smoke-test.md docs/dispatch-smoke-test.md
git commit -m "chore(dispatch): remove smoke-test validation artifacts"
git push origin master
```

Do the same for whatever other validation task files/artifacts Steps 6-7 created.

- [ ] **Step 9: Record what was validated**

In the same commit or a follow-up doc note (your call — this is validation record-keeping,
not product scope), note: which agents were actually validated end-to-end (Codex confirmed;
Antigravity confirmed or deferred with the reason why), and that the file-conflict skip
behaved correctly.

## Self-Review Notes

**Spec coverage:** §1 scope (the whole plan) ✓. §2 decisions — headless CLIs (Task 5),
on-demand trigger (no daemon anywhere in this plan) ✓, Node/TS + `tsx` (Task 1) ✓, readiness
checked not assumed (Task 2) ✓, sequential dispatch (Task 6's `for` loop, no `Promise.all`)
✓, no auto-retry (Task 6's try/catch just records the error) ✓. §3 `files:` frontmatter field
(Task 1, Step 8) ✓. §4 dispatch round steps (Tasks 2, 4, 5, 6 in sequence match exactly) ✓.
§5 error handling (Task 6's try/catch; blast-radius containment is existing sub-project-1
infrastructure, nothing new to build) ✓. §6 validation (Task 7, including the file-conflict
check) ✓. §7 follow-ups (recorded, not built — Antigravity validation in Task 7 explicitly
allowed to defer to a follow-up if install friction repeats sub-project 1's `gh` experience).

**Placeholder scan:** no TBD/TODO; every step has literal code, commands, or fully-specified
conditional instructions (e.g. Task 1 Step 1's tsconfig check names both branches explicitly).

**Type/name consistency:** `Task` interface (Task 1) used identically by `readiness.ts`,
`worktree.ts`, `invoke.ts`, `report.ts`, `index.ts`. `InvocationResult` (Task 5) matches what
`report.ts` (Task 6) expects. `ReadinessResult` (Task 2) matches what `index.ts` (Task 6)
destructures (`ready`, `reasons`). Branch format `agent/${owner}/${id}-${slug}` identical in
`worktree.ts` and the spec's §4.
