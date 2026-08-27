import type { Route } from '../../hooks/useNavigation';

export interface Stage3StubRouteMenuProps {
  onNavigate: (route: Route) => void;
  onEditGoal: () => void;
}

/** Development-only access to page stubs that are intentionally not permanent navigation. */
export function Stage3StubRouteMenu({
  onNavigate,
  onEditGoal,
}: Stage3StubRouteMenuProps) {
  if (!import.meta.env.DEV) return null;

  const routes: Array<{ label: string; route: Route }> = [
    { label: 'First launch', route: { type: 'firstLaunch' } },
    { label: 'Study session', route: { type: 'studySession', sessionId: 'session-1' } },
    {
      label: 'Full visualization',
      route: { type: 'fullVisualization', sessionId: 'session-1' },
    },
    {
      label: 'Concept view',
      route: { type: 'conceptView', workspaceId: 'calculus', conceptId: 'shell-method' },
    },
    {
      label: 'Module detail',
      route: { type: 'moduleDetail', moduleId: 'visualizer', forWorkspaceId: 'calculus' },
    },
  ];

  return (
    <details>
      <summary>Page stubs</summary>
      {routes.map(({ label, route }) => (
        <button type="button" key={label} onClick={() => onNavigate(route)}>
          {label}
        </button>
      ))}
      <button type="button" onClick={onEditGoal}>
        Goal editing
      </button>
    </details>
  );
}
