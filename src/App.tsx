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
import { useExploreSampleWorkspace } from './hooks/useExploreSampleWorkspace';
import { useKeyboardShortcut } from './hooks/useKeyboardShortcut';
import { useCommandPalette } from './hooks/useCommandPalette';
import { useNavigation, type Route } from './hooks/useNavigation';
import { WorkspaceProvider } from './hooks/WorkspaceProvider';
import { useWorkspace } from './hooks/useWorkspace';
import { useWorkspaces } from './hooks/useWorkspaces';
import { AppShell } from './layouts/AppShell';
import { RouteContent } from './layouts/RouteContent';
import { DevGalleryPage } from './pages/DevGalleryPage';
import { GoalEditingSheet } from './pages/GoalEditingSheet';
import styles from './App.module.css';

const FALLBACK_WORKSPACE_ID = 'workspace-calculus-ii';

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
  const { workspaces, refresh: refreshWorkspaces } = useWorkspaces();
  const sampleWorkspace = useExploreSampleWorkspace();
  const workspaceId = activeWorkspaceId ?? workspaces[0]?.id ?? FALLBACK_WORKSPACE_ID;
  const palette = useCommandPalette(workspaceId);

  // Workspace creation and sample import each happen through their own page-level hook
  // instance (FirstLaunchPage, CreateWorkspacePage, or this sidebar's own sampleWorkspace
  // hook) — none of them share this component's `useWorkspaces()` cache. Refetching on
  // arrival at Home is what makes the sidebar (which persists across routes, unlike a page)
  // reflect a workspace created or imported through any of those paths.
  useEffect(() => {
    if (route.type === 'home') void refreshWorkspaces();
  }, [route.type, refreshWorkspaces]);

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
      workspaces={workspaces}
      openWorkspaceId={activeWorkspaceId ?? undefined}
      activeSubItem={activeSubItem(route)}
      onSearch={palette.open}
      onHome={() => navigate({ type: 'home' })}
      onMarketplace={() => navigate({ type: 'marketplace', forWorkspaceId: workspaceId })}
      onCreateWorkspace={() => navigate({ type: 'createWorkspace' })}
      onExploreSample={sampleWorkspace.explore}
      exploringSample={sampleWorkspace.importing}
      exploreSampleError={sampleWorkspace.error}
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
        scopeLabel={workspaces.find((workspace) => workspace.id === workspaceId)?.name}
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
