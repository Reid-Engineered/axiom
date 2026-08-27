import type { ReactNode } from 'react';

import { AppShell } from '../layouts/AppShell';

export type HomePageVariant = 'default' | 'session-intent' | 'library';

export interface HomePageProps {
  variant?: HomePageVariant;
  sidebar?: ReactNode;
}

/** Home context and workspace entry points in one of the three specified variants. */
export function HomePage({ sidebar }: HomePageProps) {
  return <AppShell sidebar={sidebar}>{null}</AppShell>;
}
