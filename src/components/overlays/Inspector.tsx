import type { ReactNode } from 'react';

/**
 * Right-side inspector panel — e.g. Full Visualization's Selected shell inspector.
 * Appears only on selection and is dismissible (screen 6).
 */
export interface InspectorProps {
  open: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
  className?: string;
}

export function Inspector(_props: InspectorProps) {
  return null;
}
