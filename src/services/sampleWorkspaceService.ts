import { invoke } from '@tauri-apps/api/core';

import type { Workspace } from '../types';
import { mockConcepts } from './mockData/concepts';
import { mockGoals } from './mockData/goals';
import { mockMaterialResults, mockMaterials } from './mockData/material';
import { mockModules, mockWorkspaceTemplates } from './mockData/modules';
import { mockNotes } from './mockData/notes';
import { mockSessions } from './mockData/sessions';
import { mockWorkspaceActivity } from './mockData/workspaceActivity';
import { mockWorkspaces } from './mockData/workspaces';

const sampleWorkspaceId = 'workspace-calculus-ii';

/** Imports the retained sample fixtures into SQLite and returns the workspace to open. */
export async function importSampleWorkspace(): Promise<Workspace> {
  return invoke<Workspace>('importSampleWorkspace', {
    seed: {
      sampleWorkspaceId,
      workspaces: mockWorkspaces,
      workspaceActivity: mockWorkspaceActivity,
      goals: mockGoals,
      concepts: mockConcepts,
      modules: mockModules,
      workspaceTemplates: mockWorkspaceTemplates,
      sessions: mockSessions,
      materials: mockMaterials,
      materialResults: mockMaterialResults,
      notes: mockNotes,
    },
  });
}
