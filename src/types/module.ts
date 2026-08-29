import type { OfflineStatus, TrustLevel } from './common';

/**
 * A capability enabled inside a workspace (Tutor, Visualizer, Practice, CAS, Notes,
 * Review). Never a navigation destination; never described in developer language
 * (AXIOM-HANDOFF.md §1, invariant 5).
 */
export interface Module {
  id: string;
  name: string;
  icon: string;
  /** Built-in modules may carry no badge at all. */
  trust?: TrustLevel;
  /**
   * Human detail paired with the trust badge in compact contexts (grid/list rows), e.g.
   * "4.8k learners" or "Updated last week" — one line, one fact. Module Detail (screen 10)
   * needs both facts at once as separate metadata rows; use `lastUpdatedLabel` and
   * `learnerCountLabel` there instead of overloading this field.
   */
  trustDetail?: string;
  /** "Updated last week" — screen 10's metadata row. Distinct from `trustDetail`. */
  lastUpdatedLabel?: string;
  /** "4.8k learners" — screen 10's metadata row. Distinct from `trustDetail`. */
  learnerCountLabel?: string;
  developer: string;
  price?: string;
  /**
   * What it does for learning (screen 8's per-row description line; screen 10's first of
   * two "What it adds to your learning" paragraphs).
   */
  description: string;
  /**
   * Screen 10's second "What it adds to your learning" paragraph — the verified-primitives
   * promise in plain language. Undefined outside Module Detail's fuller copy.
   */
  learningValueDetail?: string;
  /** What context it sees, stated as capability, never a permissions list (screen 10). */
  contextSeen: string;
  offlineStatus: OfflineStatus;
  supportedConceptNames?: string[];
  worksWithModuleIds?: string[];
  suits?: string[];
  /** "What it can see" sentences, e.g. "Your notes — off by default" (screen 10). */
  privacyNotes?: string[];
  enabled: boolean;
  /** Where it appears: workspace tiles / contextual surfacing / off (screen 21). */
  visibility: 'workspace' | 'contextual' | 'off';
}

/**
 * A pre-bundled set of modules the Marketplace offers as a starting point for a workspace
 * (screen 9's "Workspace Templates" row, e.g. "Visual Learner," "Exam Intensive"). Distinct
 * from `Module` — a template isn't installed itself, it installs the modules it bundles.
 */
export interface WorkspaceTemplate {
  id: string;
  name: string;
  description: string;
  /** e.g. "4 tools" — the count shown on the template's card, not the module ids. */
  toolCount: number;
}
