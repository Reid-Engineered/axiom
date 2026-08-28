import type { ReactNode } from 'react';

import type { Route } from '../hooks/useNavigation';
import { ConceptViewPage } from '../pages/ConceptViewPage';
import { ConceptsListPage } from '../pages/ConceptsListPage';
import { CreateWorkspacePage } from '../pages/CreateWorkspacePage';
import { FirstLaunchPage } from '../pages/FirstLaunchPage';
import { FullVisualizationPage } from '../pages/FullVisualizationPage';
import { HomePage } from '../pages/HomePage';
import { MarketplacePage } from '../pages/MarketplacePage';
import { MaterialPage } from '../pages/MaterialPage';
import { ModuleDetailPage } from '../pages/ModuleDetailPage';
import { StudySessionPage } from '../pages/StudySessionPage';
import { WorkspaceOverviewPage } from '../pages/WorkspaceOverviewPage';
import { WorkspaceToolsPage } from '../pages/WorkspaceToolsPage';
import { AppShell } from './AppShell';

export interface RouteContentProps {
  route: Route;
  sidebar: ReactNode;
}

/** Maps the typed route state to its page stub and required shell. */
export function RouteContent({ route, sidebar }: RouteContentProps) {
  switch (route.type) {
    case 'firstLaunch':
      return (
        <AppShell>
          <FirstLaunchPage />
        </AppShell>
      );
    case 'createWorkspace':
      return (
        <AppShell>
          <CreateWorkspacePage />
        </AppShell>
      );
    case 'home':
      return <HomePage variant={route.variant} sidebar={sidebar} />;
    case 'workspaceOverview':
      return (
        <AppShell sidebar={sidebar}>
          <WorkspaceOverviewPage workspaceId={route.workspaceId} />
        </AppShell>
      );
    case 'studySession':
      return (
        <AppShell sidebar={sidebar}>
          <StudySessionPage sessionId={route.sessionId} />
        </AppShell>
      );
    case 'fullVisualization':
      return <FullVisualizationPage sessionId={route.sessionId} />;
    case 'conceptView':
      return (
        <AppShell sidebar={sidebar}>
          <ConceptViewPage workspaceId={route.workspaceId} conceptId={route.conceptId} />
        </AppShell>
      );
    case 'conceptsList':
      return <ConceptsListPage workspaceId={route.workspaceId} sidebar={sidebar} />;
    case 'material':
      return <MaterialPage workspaceId={route.workspaceId} sidebar={sidebar} />;
    case 'workspaceTools':
      return <WorkspaceToolsPage workspaceId={route.workspaceId} sidebar={sidebar} />;
    case 'marketplace':
      return <MarketplacePage forWorkspaceId={route.forWorkspaceId} sidebar={sidebar} />;
    case 'moduleDetail':
      return (
        <AppShell sidebar={sidebar}>
          <ModuleDetailPage
            moduleId={route.moduleId}
            forWorkspaceId={route.forWorkspaceId}
          />
        </AppShell>
      );
  }
}
