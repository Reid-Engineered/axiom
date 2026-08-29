export type MaterialResultKind = 'section' | 'workedExample' | 'exerciseRange';

/**
 * One search result within a workspace's material (screen 18). Always typed and always
 * carries the reason it matters to this learner — never a generic snippet
 * (AXIOM-HANDOFF.md §2 copy rules).
 */
export interface MaterialResult {
  id: string;
  kind: MaterialResultKind;
  page: number;
  /**
   * e.g. "§7.3 · Volumes by Cylindrical Shells", "Example 3 · region revolved about x = 2",
   * "Exercises 7.3 · 21–34".
   */
  title: string;
  /** Quoted excerpt (section) or the learner-specific reason this result surfaced. */
  reason: string;
  conceptId: string;
  /** Present if the learner has highlighted this passage. */
  highlightedAt?: string;
  /** `exerciseRange` kind only — "14 exercises on non-zero axes · 3 attempted". */
  exerciseTotal?: number;
  exerciseAttempted?: number;
}

export type ChapterSegmentStatus = 'read' | 'inProgress' | 'next' | 'outOfSyllabus';

/**
 * One block of "Where you are in the book" — always rendered left to right, oldest to
 * newest (screen 18). Out-of-syllabus chapters stay present but never drive recommendations
 * or search-first results (AXIOM-HANDOFF.md, screen 18).
 */
export interface ChapterSegment {
  /** "Ch 6–7", "Ch 8", "Ch 10–11", "Ch 12–18". */
  label: string;
  status: ChapterSegmentStatus;
  /** e.g. "33 sections" — omitted where the handoff doesn't show a count for this status. */
  detail?: string;
}

/**
 * One workspace's textbook. Material is reached through concepts and search only — there
 * is no folder view (AXIOM-HANDOFF.md, screen 18). Search results are fetched separately
 * (see `MaterialResult`) since they're query-dependent; this type is the book-level state
 * that's always loaded regardless of query.
 */
export interface Material {
  id: string;
  workspaceId: string;
  title: string;
  edition: string;
  totalPages: number;
  totalChapters: number;
  /** Always four segments, left to right. */
  segments: ChapterSegment[];
  highlightsCount: number;
  notesCount: number;
  /** "Most marked: §7.3, §8.2, §11.4". */
  mostMarkedSections: string[];
}
