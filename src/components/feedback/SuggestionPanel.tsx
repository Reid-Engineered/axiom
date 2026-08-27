/**
 * Grey panel proposing one suggestion justified by observed behaviour, with an accept
 * and a dismiss action (screens 4, 8).
 */
export interface SuggestionPanelProps {
  message: string;
  onAccept: () => void;
  acceptLabel: string;
  onDismiss: () => void;
  dismissLabel?: string;
  className?: string;
}

export function SuggestionPanel(_props: SuggestionPanelProps) {
  return null;
}
