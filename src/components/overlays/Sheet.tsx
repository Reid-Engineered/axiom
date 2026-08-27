import type { ReactNode } from 'react';

/**
 * A sheet over the dimmed workspace, not a settings page — e.g. Goal Editing's 560px
 * sheet (screen 11). 95% white, hairline border, large soft shadow, always dismissible.
 */
export interface SheetProps {
  open: boolean;
  onClose: () => void;
  eyebrow?: string;
  title?: string;
  children: ReactNode;
  footer?: ReactNode;
  className?: string;
}

export function Sheet(_props: SheetProps) {
  return null;
}
