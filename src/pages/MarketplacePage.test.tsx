import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { useNavigation } from '../hooks/useNavigation';
import { MarketplacePage } from './MarketplacePage';

function RouteObserver() {
  const { route } = useNavigation();
  return <output aria-label="Current route">{route.type}</output>;
}

function renderPage() {
  render(
    <NavigationProvider
      initialRoute={{ type: 'marketplace', forWorkspaceId: 'workspace-calculus-ii' }}
    >
      <MarketplacePage forWorkspaceId="workspace-calculus-ii" />
      <RouteObserver />
    </NavigationProvider>,
  );
}

describe('MarketplacePage', () => {
  it('renders the personalized hero, templates, trusted module grid, and quiet local row', async () => {
    renderPage();

    expect(
      await screen.findByRole('heading', { name: 'For your Calculus II workspace' }),
    ).toBeVisible();
    expect(screen.getByRole('heading', { name: 'Interactive Calculus Visualizer' })).toBeVisible();
    expect(screen.getAllByText('Axiom Verified')).toHaveLength(2);
    expect(screen.getByRole('heading', { name: 'Calculus II — Visual Learner' })).toBeVisible();
    expect(screen.getByText('7 tools')).toBeVisible();
    expect(screen.getByRole('heading', { name: 'Calculus II — Exam Intensive' })).toBeVisible();
    expect(screen.getByText('5 tools')).toBeVisible();

    const modules = screen.getByRole('heading', { name: 'Modules' }).closest('section');
    expect(within(modules!).getByRole('heading', { name: 'Proof Assistant' })).toBeVisible();
    expect(
      within(modules!).getByRole('heading', { name: 'Series Intuition Pack' }).closest('article'),
    ).toHaveTextContent(/Community.*4\.8k learners/);
    expect(within(modules!).getByText(/Suits: Learners who prefer reduced motion/)).toBeVisible();
    expect(screen.getByRole('button', { name: 'Load local module' })).toBeVisible();
  });

  it('installs into the scoped workspace and exposes the segmented sections', async () => {
    renderPage();
    const hero = (
      await screen.findByRole('heading', { name: 'Interactive Calculus Visualizer' })
    ).closest('article');
    fireEvent.click(within(hero!).getByRole('button', { name: 'Install' }));
    await waitFor(() =>
      expect(within(hero!).getByRole('button', { name: 'Installed' })).toBeDisabled(),
    );

    fireEvent.click(screen.getByRole('tab', { name: 'Templates' }));
    expect(screen.queryByRole('heading', { name: 'Interactive Calculus Visualizer' })).toBeNull();
    expect(screen.getByRole('heading', { name: 'Workspace templates' })).toBeVisible();
  });

  it('opens the featured module detail', async () => {
    renderPage();
    const hero = (
      await screen.findByRole('heading', { name: 'Interactive Calculus Visualizer' })
    ).closest('article');
    fireEvent.click(within(hero!).getByRole('button', { name: 'Learn more' }));
    await waitFor(() =>
      expect(screen.getByLabelText('Current route')).toHaveTextContent('moduleDetail'),
    );
  });
});
