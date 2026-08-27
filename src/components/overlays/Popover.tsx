import type { ReactNode, RefObject } from 'react';

/** 95% white, hairline border, soft shadow, always dismissible, always secondary to content. */
export interface PopoverProps {
  open: boolean;
  onClose: () => void;
  anchorRef?: RefObject<HTMLElement>;
  children: ReactNode;
  className?: string;
}

export function Popover(_props: PopoverProps) {
  return null;
}
