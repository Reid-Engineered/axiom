import { useCallback, useMemo, useState } from 'react';

import { getRecentNotes } from '../services/noteService';
import type { Concept, Module, Note } from '../types';
import { useAsyncResource } from './useAsyncResource';
import { useConcepts } from './useConcepts';
import { useMarketplaceModules } from './useModules';
import { useNavigation } from './useNavigation';
import { useActiveSession } from './useSessions';

export interface CommandPaletteAction {
  id: string;
  label: string;
  detail?: string;
  shortcut?: string;
  run: () => void;
}

function matches(query: string, ...values: Array<string | undefined>) {
  const terms = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
  const searchable = values.filter(Boolean).join(' ').toLocaleLowerCase();
  return terms.every((term) => searchable.includes(term));
}

/** Coordinates Command Palette overlay state and its current query. */
export function useCommandPalette(workspaceId = 'workspace-calculus-ii') {
  const { overlay, openOverlay, closeOverlay, navigate } = useNavigation();
  const { concepts } = useConcepts(workspaceId);
  const { modules } = useMarketplaceModules(workspaceId);
  const { session } = useActiveSession(workspaceId);
  const loadNotes = useCallback(() => getRecentNotes(workspaceId), [workspaceId]);
  const noteResource = useAsyncResource(loadNotes);
  const [query, setQuery] = useState('');

  const open = useCallback(() => openOverlay({ type: 'commandPalette' }), [openOverlay]);
  const close = useCallback(() => {
    setQuery('');
    closeOverlay();
  }, [closeOverlay]);

  const activeConcept = concepts.find((concept) => concept.id === session?.conceptId);
  const conceptLabel = activeConcept?.name ?? 'current concept';
  const titleConcept = conceptLabel.replace(/\b\w/g, (letter) => letter.toUpperCase());
  const runInSession = useCallback(() => {
    if (session) navigate({ type: 'studySession', sessionId: session.id });
  }, [navigate, session]);
  const openVisualization = useCallback(() => {
    if (session) navigate({ type: 'fullVisualization', sessionId: session.id });
  }, [navigate, session]);

  const actions = useMemo<CommandPaletteAction[]>(
    () =>
      [
        {
          id: 'practice-current-concept',
          label: `Practice the ${titleConcept}`,
          detail: session?.problemCount
            ? `${session.problemCount} problems · adaptive`
            : 'Adaptive practice',
          shortcut: '⏎',
          run: runInSession,
        },
        {
          id: 'visualize-current-concept',
          label: `Visualize the ${titleConcept}`,
          detail: 'opens full view',
          run: openVisualization,
        },
        {
          id: 'ask-tutor',
          label: `Ask the tutor about the ${titleConcept}`,
          shortcut: '⌘T',
          run: runInSession,
        },
        {
          id: 'new-note',
          label: `New note on the ${titleConcept}`,
          shortcut: '⌘N',
          run: close,
        },
      ].filter((action) => matches(query, action.label, action.detail)),
    [close, openVisualization, query, runInSession, session?.problemCount, titleConcept],
  );

  const conceptResults = useMemo(() => {
    const direct = concepts.filter((concept) => matches(query, concept.name, concept.chapter));
    const queryMatchesActiveConcept =
      query.trim().length > 0 && activeConcept && matches(query, activeConcept.name);
    if (!queryMatchesActiveConcept) return direct.slice(0, 2);
    const relatedIds = new Set([
      ...activeConcept.prerequisiteConceptIds,
      ...activeConcept.relatedConceptIds,
    ]);
    const related = concepts.filter((concept) => relatedIds.has(concept.id));
    return [
      ...new Map([...direct, ...related].map((concept) => [concept.id, concept])).values(),
    ].slice(0, 2);
  }, [activeConcept, concepts, query]);

  const notes = (noteResource.data ?? []).filter((note) => matches(query, note.text));
  const marketplaceModules = modules
    .filter(
      (module) => module.trust === 'community' && matches(query, module.name, module.description),
    )
    .slice(0, 1);

  return {
    isOpen: overlay?.type === 'commandPalette',
    query,
    setQuery,
    open,
    close,
    actions,
    concepts: conceptResults as Concept[],
    notes: notes as Note[],
    marketplaceModules: marketplaceModules as Module[],
  };
}
