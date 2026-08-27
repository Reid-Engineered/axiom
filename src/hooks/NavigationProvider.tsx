import { useCallback, useMemo, useState, type ReactNode } from 'react';

import {
  NavigationContext,
  type Overlay,
  type Route,
} from './navigationContext';

export interface NavigationProviderProps {
  children: ReactNode;
  initialRoute?: Route;
}

/** Owns the window's route and optional overlay state. */
export function NavigationProvider({
  children,
  initialRoute = { type: 'firstLaunch' },
}: NavigationProviderProps) {
  const [route, setRoute] = useState<Route>(initialRoute);
  const [overlay, setOverlay] = useState<Overlay | null>(null);

  const navigate = useCallback((nextRoute: Route) => {
    setRoute(nextRoute);
    setOverlay(null);
  }, []);
  const openOverlay = useCallback((nextOverlay: Overlay) => setOverlay(nextOverlay), []);
  const closeOverlay = useCallback(() => setOverlay(null), []);

  const value = useMemo(
    () => ({ route, overlay, navigate, openOverlay, closeOverlay }),
    [route, overlay, navigate, openOverlay, closeOverlay],
  );

  return <NavigationContext.Provider value={value}>{children}</NavigationContext.Provider>;
}
