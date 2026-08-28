import { useCallback, useState } from 'react';

import { useNavigation } from './useNavigation';

/** Coordinates Command Palette overlay state and its current query. */
export function useCommandPalette() {
  const { overlay, openOverlay, closeOverlay } = useNavigation();
  const [query, setQuery] = useState('');

  const open = useCallback(() => openOverlay({ type: 'commandPalette' }), [openOverlay]);
  const close = useCallback(() => {
    setQuery('');
    closeOverlay();
  }, [closeOverlay]);

  return {
    isOpen: overlay?.type === 'commandPalette',
    query,
    setQuery,
    open,
    close,
  };
}
