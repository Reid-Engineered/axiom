import { createContext } from 'react';

import type { HomePageVariant } from '../pages/HomePage';

export type Route =
  | { type: 'firstLaunch' }
  | { type: 'createWorkspace' }
  | { type: 'home'; variant?: HomePageVariant }
  | { type: 'workspaceOverview'; workspaceId: string }
  | { type: 'studySession'; sessionId: string }
  | { type: 'fullVisualization'; sessionId: string }
  | { type: 'conceptView'; workspaceId: string; conceptId: string }
  | { type: 'conceptsList'; workspaceId: string }
  | { type: 'material'; workspaceId: string }
  | { type: 'workspaceTools'; workspaceId: string }
  | { type: 'marketplace'; forWorkspaceId?: string }
  | { type: 'moduleDetail'; moduleId: string; forWorkspaceId?: string };

export type Overlay =
  | { type: 'commandPalette' }
  | { type: 'goalEditing'; workspaceId: string; goalId: string };

export interface NavigationContextValue {
  route: Route;
  overlay: Overlay | null;
  navigate: (route: Route) => void;
  openOverlay: (overlay: Overlay) => void;
  closeOverlay: () => void;
}

export const NavigationContext = createContext<NavigationContextValue | null>(null);
