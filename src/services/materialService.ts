import { invoke } from '@tauri-apps/api/core';

import type { Material, MaterialResult } from '../types';

export async function getMaterial(workspaceId: string): Promise<Material> {
  return invoke<Material>('getMaterial', { workspaceId });
}

/** Searches only in-syllabus concept-linked material; browse-only chapters remain excluded. */
export async function searchMaterial(
  workspaceId: string,
  query: string,
): Promise<MaterialResult[]> {
  return invoke<MaterialResult[]>('searchMaterial', { workspaceId, query });
}
