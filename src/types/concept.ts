import type { MasteryState } from './common';

/**
 * A single diagnosed practice attempt, surfaced as a DiagnosticDot (concept view's
 * "Recent practice", screen 7).
 */
export interface ConceptDiagnostic {
  id: string;
  expression: string;
  type: 'mistake' | 'positive' | 'neutral';
  note: string;
  occurredAt: string;
}

/**
 * Semantic node with prerequisite/related/leads-to edges, a mastery state, and links
 * to notes, practice, visualizations, tutor threads (AXIOM-HANDOFF.md §1).
 */
export interface Concept {
  id: string;
  workspaceId: string;
  name: string;
  /** Grouping label matching the textbook's chapter structure, e.g. "7 · Applications of Integration". */
  chapter: string;
  masteryState: MasteryState;
  /** Decayed state, if any — "was Strong" (screen 16, 17). */
  wasMasteryState?: MasteryState;
  decayedAt?: string;
  /** One-line meaning of the current state, e.g. "held up weeks apart without review". */
  meaning: string;
  dueForReviewInDays?: number;
  onExam: boolean;
  /** Concepts this one blocks — drives "blocks 3 concepts" and needs-work ordering. */
  blocksConceptIds: string[];
  prerequisiteConceptIds: string[];
  relatedConceptIds: string[];
  leadsToConceptIds: string[];
  displayFormula?: string;
  explanation?: string;
  /** The learner's own heuristic, quoted back with evidence (screen 7). */
  learnerHeuristic?: string;
  heuristicEvidence?: string;
  whereItShowsUp?: string[];
  recentDiagnostics?: ConceptDiagnostic[];
  lastActivityAt?: string;
  notesCount: number;
}
