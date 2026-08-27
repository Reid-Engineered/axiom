import type { MathSegment } from './MathDisplay';

/**
 * Inline math set in STIX Two Text — variables italic, operators upright
 * (`AGENTS.md` UI rules, `AXIOM-HANDOFF.md` §2). Same segmented shape as `MathDisplay`
 * for consistency, though today's screens only need term selection in the display well.
 */
export interface MathInlineProps {
  expression: string | MathSegment[];
  onSelectTerm?: (segment: MathSegment) => void;
  className?: string;
}

export function MathInline(_props: MathInlineProps) {
  return null;
}
