/**
 * The four rows the "Make available offline" sheet toggles independently
 * (screenshots/21-offline-modules-goals.png). "Visual assets & module data" is one
 * combined toggle, not two.
 */
export type OfflineContentKind =
  | 'textbookAndLectureNotes'
  | 'problemBanks'
  | 'visualAssetsAndModuleData'
  | 'courseVideos';

export interface OfflineKindAvailability {
  kind: OfflineContentKind;
  enabled: boolean;
  sizeBytes: number;
  /**
   * Present when only part of this kind can be made available — screen 21's Course
   * videos row ("9 of 32 downloadable — the rest are streamed by your school").
   * `limitReason` is stated in the learner's terms, never as an error.
   */
  partial?: {
    availableCount: number;
    totalCount: number;
    limitReason: string;
  };
}

/**
 * A curated learning environment for one subject. Owns goals, concepts, material,
 * history, enabled modules, and learning preferences (AXIOM-HANDOFF.md §1).
 */
export interface Workspace {
  id: string;
  name: string;
  guidingGoalId: string;
  /** 0-1 fraction driving the unlabelled ProgressBar fill — never rendered as text. */
  progress: number;
  lastConceptName?: string;
  lastActivityAt?: string;
  paused: boolean;
  /**
   * Per-kind offline toggle state. The toolbar's single "Available offline" chip and
   * the sheet's total size are both derived from this, not stored separately.
   */
  offlineAvailability: OfflineKindAvailability[];
  enabledModuleIds: string[];
}
