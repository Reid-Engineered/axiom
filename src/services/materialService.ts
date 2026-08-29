import type { Material, MaterialResult } from '../types';
import { mockMaterialResults, mockMaterials } from './mockData/material';

export async function getMaterial(workspaceId: string): Promise<Material> {
  const material = mockMaterials.find((candidate) => candidate.workspaceId === workspaceId);
  if (!material) throw new Error(`Material not found for workspace: ${workspaceId}`);
  return structuredClone(material);
}

/** Searches only in-syllabus concept-linked material; browse-only chapters remain excluded. */
export async function searchMaterial(
  workspaceId: string,
  query: string,
): Promise<MaterialResult[]> {
  const material = mockMaterials.find((candidate) => candidate.workspaceId === workspaceId);
  if (!material) throw new Error(`Material not found for workspace: ${workspaceId}`);

  const terms = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  const results = mockMaterialResults.filter((result) => {
    if (!result.inSyllabus) return false;
    const searchable = `${result.title} ${result.reason}`.toLocaleLowerCase();
    return terms.every((term) => searchable.includes(term));
  });
  return structuredClone(results);
}
