import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { useNavigation } from '../hooks/useNavigation';
import { ConceptViewPage } from './ConceptViewPage';

function RouteObserver() {
  const { route } = useNavigation();
  return <output aria-label="Current route">{route.type}</output>;
}

function renderPage() {
  render(
    <NavigationProvider
      initialRoute={{
        type: 'conceptView',
        workspaceId: 'workspace-calculus-ii',
        conceptId: 'calc-concept-14',
      }}
    >
      <ConceptViewPage workspaceId="workspace-calculus-ii" conceptId="calc-concept-14" />
      <RouteObserver />
    </NavigationProvider>,
  );
}

describe('ConceptViewPage', () => {
  it('composes the real concept detail, mastery, formula, edges, diagnostics, and notes', async () => {
    renderPage();
    const title = await screen.findByRole('heading', { name: 'Integration by parts', level: 1 });
    expect(title).toBeVisible();
    expect(within(title.closest('header')!).getByText('Mastered')).toBeVisible();
    expect(screen.getByText('Held up weeks apart without review.')).toBeVisible();
    expect(screen.getByText('Due for review in 2 days')).toBeVisible();
    expect(screen.getByText('∫ u dv = uv − ∫ v du')).toBeVisible();

    const buildsOn = screen.getByRole('heading', { name: 'Builds on' }).closest('section');
    const leadsTo = screen.getByRole('heading', { name: 'Leads to' }).closest('section');
    expect(within(buildsOn!).getByRole('button', { name: /Product rule/ })).toBeVisible();
    expect(within(buildsOn!).getByRole('button', { name: /Substitution/ })).toBeVisible();
    expect(within(leadsTo!).getByRole('button', { name: /Trigonometric integrals/ })).toBeVisible();
    expect(screen.getByText(/Chose u backwards/)).toBeVisible();
    expect(screen.getByText('Taylor remainder')).toBeVisible();
    expect(screen.getByText('3 notes')).toBeVisible();
    expect(screen.getByRole('button', { name: '2 more notes' })).toBeVisible();
    expect(screen.getAllByText(/Differentiate the messy part/)).toHaveLength(1);
  });

  it('starts a fixture-backed practice session and navigates to it', async () => {
    renderPage();
    fireEvent.click(await screen.findByRole('button', { name: 'Practice this' }));
    await waitFor(() =>
      expect(screen.getByLabelText('Current route')).toHaveTextContent('studySession'),
    );
  });
});
