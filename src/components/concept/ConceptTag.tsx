/**
 * A single concept reference tag — Concept View's "Where it shows up" and Module
 * Detail's supported-concepts list (screens 7, 10). Not removable, unlike primitives/Chip.
 */
export interface ConceptTagProps {
  label: string;
  onSelect?: () => void;
  className?: string;
}

export function ConceptTag(_props: ConceptTagProps) {
  return null;
}
