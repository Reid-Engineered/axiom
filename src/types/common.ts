/**
 * Fixed enums shared across the domain model. See AXIOM-HANDOFF.md §1.
 */

/** Five named mastery states. The word is the indicator — never a percentage. */
export type MasteryState = 'New' | 'Developing' | 'Familiar' | 'Strong' | 'Mastered';

/** A workspace- or resource-level offline promise. */
export type OfflineStatus = 'Works offline' | 'Online enhanced' | 'Internet required';

/** Module provenance, shown via TrustBadge. */
export type TrustLevel = 'verified' | 'community' | 'experimental';

/** Exactly one Goal per workspace is 'Guiding' at a time (AXIOM-HANDOFF.md §1). */
export type GoalState = 'Guiding' | 'Waiting' | 'Met' | 'Resting';
