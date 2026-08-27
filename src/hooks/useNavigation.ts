import { useContext } from 'react';

import { NavigationContext, type NavigationContextValue } from './navigationContext';

export type { Overlay, Route } from './navigationContext';

/** Reads route and overlay navigation state. */
export function useNavigation(): NavigationContextValue {
  const value = useContext(NavigationContext);

  if (!value) {
    throw new Error('useNavigation must be used within NavigationProvider');
  }

  return value;
}
