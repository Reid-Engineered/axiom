import { useState } from 'react';

import { CommandPalette } from './CommandPalette';

export interface StubCommandPaletteProps {
  open: boolean;
  onClose: () => void;
}

/** Empty-results Command Palette used until Stage 6 supplies real grouped results. */
export function StubCommandPalette({ open, onClose }: StubCommandPaletteProps) {
  const [query, setQuery] = useState('');

  return (
    <CommandPalette
      open={open}
      onClose={onClose}
      query={query}
      onQueryChange={setQuery}
      groups={[]}
      scopeLabel="Calculus II"
    />
  );
}
