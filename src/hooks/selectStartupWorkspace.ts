import type { Workspace } from '../types';

function compareNullableTimestampDescending(left?: string, right?: string) {
  if (left && right) return right.localeCompare(left);
  if (left) return -1;
  if (right) return 1;
  return 0;
}

/** Selects the deterministic workspace whose domain activity should define startup context. */
export function selectStartupWorkspace(workspaces: Workspace[]): Workspace | undefined {
  const everyActivityIsNull = workspaces.every((workspace) => !workspace.lastActivityAt);

  return [...workspaces].sort((left, right) => {
    const activityOrder = compareNullableTimestampDescending(
      left.lastActivityAt,
      right.lastActivityAt,
    );
    if (activityOrder !== 0) return activityOrder;

    if (everyActivityIsNull) {
      const creationOrder = compareNullableTimestampDescending(left.createdAt, right.createdAt);
      if (creationOrder !== 0) return creationOrder;
    }

    return right.id.localeCompare(left.id);
  })[0];
}
