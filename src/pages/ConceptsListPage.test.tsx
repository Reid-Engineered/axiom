import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { useNavigation } from '../hooks/useNavigation';
import { ConceptsListPage } from './ConceptsListPage';

function RouteObserver() {
  const { route } = useNavigation();
  return <output aria-label="Current route">{route.type}</output>;
}

function renderPage() {
  return render(
    <NavigationProvider
      initialRoute={{ type: 'conceptsList', workspaceId: 'workspace-calculus-ii' }}
    >
      <ConceptsListPage workspaceId="workspace-calculus-ii" />
      <RouteObserver />
    </NavigationProvider>,
  );
}

describe('ConceptsListPage', () => {
  it('demonstrates the 87-concept scale without opening the list flat', async () => {
    const { container } = renderPage();

    expect(await screen.findByText('87 in this workspace')).toBeVisible();
    expect(screen.getByPlaceholderText('Search 87 concepts')).toBeVisible();

    // Verify all 6 actionable filter chips are present
    expect(screen.getByRole('button', { name: 'Needs work · 6' })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    expect(screen.getByRole('button', { name: /Due for review ·/ })).toHaveAttribute(
      'aria-pressed',
      'false',
    );
    expect(screen.getByRole('button', { name: /In progress ·/ })).toHaveAttribute(
      'aria-pressed',
      'false',
    );
    expect(screen.getByRole('button', { name: /On the exam ·/ })).toHaveAttribute(
      'aria-pressed',
      'false',
    );
    expect(screen.getByRole('button', { name: /Not started ·/ })).toHaveAttribute(
      'aria-pressed',
      'false',
    );
    expect(screen.getByRole('button', { name: 'All' })).toHaveAttribute('aria-pressed', 'false');

    const needsWork = screen.getByRole('heading', { name: 'Needs work' }).closest('section');
    expect(within(needsWork!).getAllByTitle(/Mastery:/)).toHaveLength(3);
    expect(within(needsWork!).getByText('ordered by what it blocks')).toBeVisible();
    expect(within(needsWork!).getByRole('button', { name: '3 more need work' })).toBeVisible();

    const chapters = Array.from(container.querySelectorAll('details'));
    expect(chapters).toHaveLength(9);
    expect(chapters.every((chapter) => !chapter.open)).toBe(true);
    expect(screen.getByText('1 · Review of Functions')).toBeVisible();
    expect(screen.getAllByText(/concepts?$/).length).toBeGreaterThanOrEqual(9);
  });

  it('includes needs-work concepts inside their chapter groups without exclusion', async () => {
    renderPage();
    await screen.findByText('87 in this workspace');

    // Get the top needs-work concept name
    const needsWork = screen.getByRole('heading', { name: 'Needs work' }).closest('section');
    const firstNeedsWorkRow = within(needsWork!).getAllByRole('button')[0];
    const conceptName = firstNeedsWorkRow.textContent?.split('blocks')[0].trim();
    expect(conceptName).toBeTruthy();

    // Verify this concept also exists in one of the chapter detail sections
    const chaptersSection = screen.getByRole('heading', { name: 'By chapter' }).closest('section');
    const chapterButtons = within(chaptersSection!).getAllByRole('button');
    const matchingInChapters = chapterButtons.filter((btn) =>
      btn.textContent?.includes(conceptName!),
    );
    expect(matchingInChapters.length).toBeGreaterThanOrEqual(1);
  });

  it('expands needs work, filters by in-progress and not-started, and keeps graph opt-in', async () => {
    renderPage();
    await screen.findByText('87 in this workspace');

    const needsWork = screen.getByRole('heading', { name: 'Needs work' }).closest('section');
    fireEvent.click(within(needsWork!).getByRole('button', { name: '3 more need work' }));
    expect(within(needsWork!).getAllByTitle(/Mastery:/)).toHaveLength(6);

    const inProgressFilter = screen.getByRole('button', { name: /In progress ·/ });
    fireEvent.click(inProgressFilter);
    expect(inProgressFilter).toHaveAttribute('aria-pressed', 'true');

    const notStartedFilter = screen.getByRole('button', { name: /Not started ·/ });
    fireEvent.click(notStartedFilter);
    expect(notStartedFilter).toHaveAttribute('aria-pressed', 'true');

    const dueFilter = screen.getByRole('button', { name: /Due for review ·/ });
    fireEvent.click(dueFilter);
    expect(dueFilter).toHaveAttribute('aria-pressed', 'true');

    const graph = screen.getByRole('button', { name: 'Graph' });
    expect(graph).toHaveAttribute('aria-pressed', 'false');
    expect(screen.queryByText('Concept graph')).not.toBeInTheDocument();
    fireEvent.click(graph);
    expect(screen.getByText('Concept graph')).toBeVisible();
  });

  it('opens a concept through the shared ConceptRow', async () => {
    renderPage();
    const needsWork = (await screen.findByRole('heading', { name: 'Needs work' })).closest(
      'section',
    );
    const concept = within(needsWork!).getAllByRole('button')[0];
    fireEvent.click(concept);
    await waitFor(() =>
      expect(screen.getByLabelText('Current route')).toHaveTextContent('conceptView'),
    );
  });
});
