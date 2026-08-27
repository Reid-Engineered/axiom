import { AppShell } from '../layouts/AppShell';

export type HomePageVariant = 'default' | 'session-intent' | 'library';

export interface HomePageProps {
  variant?: HomePageVariant;
}

/** Home context and workspace entry points in one of the three specified variants. */
export function HomePage(_props: HomePageProps) {
  return <AppShell>{null}</AppShell>;
}
