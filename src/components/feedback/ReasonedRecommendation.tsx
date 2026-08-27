export interface ReasonedRecommendationObservation {
  date: string;
  text: string;
}

/**
 * Accent-ruled block: action, one line of evidence, a primary CTA, and an optional
 * "Why this?" expanding to dated observations. Never mentions AI, never explains the
 * model (screen 4, `AXIOM-HANDOFF.md` §2).
 */
export interface ReasonedRecommendationProps {
  action: string;
  evidence: string;
  ctaLabel: string;
  onStart: () => void;
  observations?: ReasonedRecommendationObservation[];
  onAlternative?: () => void;
  className?: string;
}

export function ReasonedRecommendation(_props: ReasonedRecommendationProps) {
  return null;
}
