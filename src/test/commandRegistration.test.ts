import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

/**
 * Every command the frontend invokes must be reachable in a real build.
 *
 * The frontend suite runs against `mockBackend.ts` and the Rust suite never crosses the
 * IPC boundary, so a service calling a command that was never registered in `lib.rs`
 * passes both — and fails only when a human opens the app. That happened: task 066
 * merged `describeAttempt` and `nextProblem` callers while task 065, which registers
 * them, was still in review, and all seven required checks stayed green.
 */

const ROOT = process.cwd();
const SERVICES_DIR = join(ROOT, 'src/services');
const COMMANDS_DIR = join(ROOT, 'src-tauri/src/commands');
const LIB_RS = join(ROOT, 'src-tauri/src/lib.rs');

/** Command names passed to `invoke(...)` anywhere under `src/services/`. */
function invokedCommandNames(): Map<string, string> {
  const found = new Map<string, string>();
  for (const file of readdirSync(SERVICES_DIR).filter((name) => name.endsWith('.ts'))) {
    const source = readFileSync(join(SERVICES_DIR, file), 'utf8');
    for (const match of source.matchAll(/invoke(?:<[^>]*>)?\(\s*'([^']+)'/g)) {
      found.set(match[1], file);
    }
  }
  return found;
}

/** Rust fn names listed inside `tauri::generate_handler![...]` in `lib.rs`. */
function registeredHandlerFns(): Set<string> {
  const source = readFileSync(LIB_RS, 'utf8');
  const block = source.match(/generate_handler!\[([\s\S]*?)\]/);
  if (!block) throw new Error('could not find generate_handler![...] in lib.rs');
  return new Set(
    block[1]
      .split(',')
      .map((entry) => entry.trim().split('::').pop() ?? '')
      .filter(Boolean),
  );
}

/**
 * Maps the IPC name a command is exposed under to its Rust fn name. Commands declare
 * the name explicitly via `#[tauri::command(rename = "...")]`; without a rename, Tauri
 * uses the fn name itself.
 */
function commandNameToFn(): Map<string, string> {
  const mapping = new Map<string, string>();
  for (const file of readdirSync(COMMANDS_DIR).filter((name) => name.endsWith('.rs'))) {
    const source = readFileSync(join(COMMANDS_DIR, file), 'utf8');
    const pattern =
      /#\[tauri::command(?:\(([^)]*)\))?\]\s*pub\s+(?:async\s+)?fn\s+([a-z0-9_]+)/g;
    for (const match of source.matchAll(pattern)) {
      const [, args = '', fnName] = match;
      const renamed = args.match(/rename\s*=\s*"([^"]+)"/);
      mapping.set(renamed ? renamed[1] : fnName, fnName);
    }
  }
  return mapping;
}

describe('IPC command registration', () => {
  const invoked = invokedCommandNames();
  const nameToFn = commandNameToFn();
  const registered = registeredHandlerFns();

  it('finds commands to check', () => {
    // Guards the regexes above: if a refactor changes how services call `invoke` or how
    // commands are declared, this fails loudly instead of vacuously passing on an empty set.
    expect(invoked.size).toBeGreaterThan(20);
    expect(nameToFn.size).toBeGreaterThan(20);
    expect(registered.size).toBeGreaterThan(20);
  });

  it('exposes every command the frontend invokes', () => {
    const missing = [...invoked.entries()]
      .filter(([name]) => !nameToFn.has(name))
      .map(([name, file]) => `${name} (called from services/${file}) — no #[tauri::command]`);
    expect(missing).toEqual([]);
  });

  it('registers every invoked command in lib.rs', () => {
    const unwired = [...invoked.entries()]
      .filter(([name]) => nameToFn.has(name) && !registered.has(nameToFn.get(name)!))
      .map(
        ([name, file]) =>
          `${name} (called from services/${file}) — declared as ${nameToFn.get(name)} but missing from generate_handler!`,
      );
    expect(unwired).toEqual([]);
  });
});
