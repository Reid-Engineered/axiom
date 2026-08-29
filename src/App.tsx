import { useEffect, useState } from 'react';

import { TrustBadge } from './components/badges/TrustBadge';
import { ConceptRow } from './components/concept/ConceptRow';
import {
  CommandPalette,
  CommandPaletteMarketplaceResult,
  CommandPaletteText,
  type CommandPaletteResultGroup,
} from './components/overlays/CommandPalette';
import { Sidebar } from './components/workspace/Sidebar';
import { Stage3StubRouteMenu } from './components/workspace/Stage3StubRouteMenu';
import type { WorkspaceTreeProps } from './components/workspace/WorkspaceTree';
import { NavigationProvider } from './hooks/NavigationProvider';
import { useKeyboardShortcut } from './hooks/useKeyboardShortcut';
import { useCommandPalette } from './hooks/useCommandPalette';
import { useNavigation, type Route } from './hooks/useNavigation';
import { WorkspaceProvider } from './hooks/WorkspaceProvider';
import { useWorkspace } from './hooks/useWorkspace';
import { AppShell } from './layouts/AppShell';
import { RouteContent } from './layouts/RouteContent';
import { DevGalleryPage } from './pages/DevGalleryPage';
import { GoalEditingSheet } from './pages/GoalEditingSheet';
import styles from './App.module.css';

const WORKSPACES = [
  { id: 'workspace-calculus-ii', name: 'Calculus II' },
  { id: 'workspace-linear-algebra', name: 'Linear Algebra' },
  { id: 'workspace-physics', name: 'Mechanics' },
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
  const palette = useCommandPalette(workspaceId);

  const commandGroups: CommandPaletteResultGroup[] = [
    {
      label: 'Actions',
      items: palette.actions.map((action) => ({
        id: action.id,
        content: <CommandPaletteText label={action.label} detail={action.detail} />,
        shortcut: action.shortcut,
        onSelect: action.run,
      })),
    },
    {
      label: 'Concepts',
      items: palette.concepts.map((concept) => ({
        id: concept.id,
        content: (
          <ConceptRow
            name={concept.name}
            masteryState={concept.masteryState}
            statusText={concept.id === palette.concepts[0]?.id ? 'active' : 'related'}
          />
        ),
        onSelect: () => navigate({ type: 'conceptView', workspaceId, conceptId: concept.id }),
      })),
    },
    {
      label: 'From your work',
      items: palette.notes.map((note) => ({
        id: note.id,
        content: <CommandPaletteText label={`Note — “${note.text}”`} />,
        onSelect: () => navigate({ type: 'conceptView', workspaceId, conceptId: note.conceptId }),
      })),
    },
    {
      label: 'Marketplace',
      items: palette.marketplaceModules.map((module) => ({
        id: module.id,
        content: (
          <CommandPaletteMarketplaceResult
            label={`Marketplace — ${module.name}`}
            badge={module.trust ? <TrustBadge type={module.trust} /> : null}
          />
        ),
        onSelect: () =>
          navigate({ type: 'moduleDetail', moduleId: module.id, forWorkspaceId: workspaceId }),
      })),
    },
  ].filter((group) => group.items.length > 0);

  useKeyboardShortcut('k', palette.open);

  const sidebar = (
    <Sidebar
      workspaces={WORKSPACES}
      openWorkspaceId={activeWorkspaceId ?? undefined}
      activeSubItem={activeSubItem(route)}
      onSearch={palette.open}
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
      <CommandPalette
        open={palette.isOpen}
        onClose={palette.close}
        query={palette.query}
        onQueryChange={palette.setQuery}
        groups={commandGroups}
        scopeLabel={WORKSPACES.find((workspace) => workspace.id === workspaceId)?.name}
      />
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
    <WorkspaceProvider>
      <NavigationProvider initialRoute={{ type: 'firstLaunch' }}>
        <Application />
      </NavigationProvider>
    </WorkspaceProvider>
  );
}

export default App;
