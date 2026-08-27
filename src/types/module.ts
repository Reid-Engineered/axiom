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
  /** Human detail paired with trust, e.g. "4.8k learners", "Updated last week". */
  trustDetail?: string;
  developer: string;
  price?: string;
  /** What it does for learning (screen 8's per-row description line). */
  description: string;
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
