import { useCallback, useEffect, useState } from 'react';

import { StubCommandPalette } from './components/overlays/StubCommandPalette';
import { Sidebar } from './components/workspace/Sidebar';
import { Stage3StubRouteMenu } from './components/workspace/Stage3StubRouteMenu';
import type { WorkspaceTreeProps } from './components/workspace/WorkspaceTree';
import { NavigationProvider } from './hooks/NavigationProvider';
import { useKeyboardShortcut } from './hooks/useKeyboardShortcut';
import { useNavigation, type Route } from './hooks/useNavigation';
import { WorkspaceProvider } from './hooks/WorkspaceProvider';
import { useWorkspace } from './hooks/useWorkspace';
import { AppShell } from './layouts/AppShell';
import { RouteContent } from './layouts/RouteContent';
import { DevGalleryPage } from './pages/DevGalleryPage';
import { GoalEditingSheet } from './pages/GoalEditingSheet';
import styles from './App.module.css';

const WORKSPACES = [
  { id: 'calculus', name: 'Calculus II' },
  { id: 'linear-algebra', name: 'Linear Algebra' },
  { id: 'circuits', name: 'Circuit Analysis' },
];

function activeSubItem(route: Route): WorkspaceTreeProps['activeSubItem'] {
  if (route.type === 'workspaceOverview') return 'overview';
  if (route.type === 'conceptsList') return 'concepts';
  if (route.type === 'material') return 'material';
  if (route.type === 'workspaceTools') return 'tools';
  return undefined;
}

function Application() {
  const { route, overlay, navigate, openOverlay, closeOverlay } = useNavigation();
  const { activeWorkspaceId, setActiveWorkspaceId } = useWorkspace();
  const workspaceId = activeWorkspaceId ?? WORKSPACES[0].id;
  const openCommandPalette = useCallback(
    () => openOverlay({ type: 'commandPalette' }),
    [openOverlay],
  );

  useKeyboardShortcut('k', openCommandPalette);

  const sidebar = (
    <Sidebar
      workspaces={WORKSPACES}
      openWorkspaceId={activeWorkspaceId ?? undefined}
      activeSubItem={activeSubItem(route)}
      onSearch={openCommandPalette}
      onHome={() => navigate({ type: 'home' })}
      onMarketplace={() => navigate({ type: 'marketplace', forWorkspaceId: workspaceId })}
      onCreateWorkspace={() => navigate({ type: 'createWorkspace' })}
      onSelectWorkspace={(selectedWorkspaceId) => {
        setActiveWorkspaceId(selectedWorkspaceId);
        navigate({ type: 'workspaceOverview', workspaceId: selectedWorkspaceId });
      }}
      onSelectSubItem={(subItem) => {
        const routeBySubItem: Record<typeof subItem, Route> = {
          overview: { type: 'workspaceOverview', workspaceId },
          concepts: { type: 'conceptsList', workspaceId },
          material: { type: 'material', workspaceId },
          tools: { type: 'workspaceTools', workspaceId },
        };
        navigate(routeBySubItem[subItem]);
      }}
      footer={
        <Stage3StubRouteMenu
          onNavigate={navigate}
          onEditGoal={() =>
            openOverlay({ type: 'goalEditing', workspaceId, goalId: 'guiding-goal' })
          }
        />
      }
    />
  );

  return (
    <div
      className={styles.routeRoot}
      data-route={route.type}
      data-overlay={overlay?.type ?? undefined}
    >
      <RouteContent route={route} sidebar={sidebar} />
      <StubCommandPalette open={overlay?.type === 'commandPalette'} onClose={closeOverlay} />
      {overlay?.type === 'goalEditing' ? (
        <GoalEditingSheet
          open
          workspaceId={overlay.workspaceId}
          goalId={overlay.goalId}
          onClose={closeOverlay}
        />
      ) : null}
    </div>
  );
}

function App() {
  const [hash, setHash] = useState(() => window.location.hash);

  useEffect(() => {
    const handleHashChange = () => setHash(window.location.hash);
    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  if (import.meta.env.DEV && hash === '#/dev/gallery') {
    return (
      <AppShell>
        <DevGalleryPage />
      </AppShell>
    );
  }

  return (
    <WorkspaceProvider initialWorkspaceId="calculus">
      <NavigationProvider initialRoute={{ type: 'home' }}>
        <Application />
      </NavigationProvider>
    </WorkspaceProvider>
  );
}

export default App;
