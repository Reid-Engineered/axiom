import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { useNavigation } from '../hooks/useNavigation';
import { MaterialPage } from './MaterialPage';

function RouteObserver() {
  const { route } = useNavigation();
  return <output aria-label="Current route">{route.type}</output>;
}

function renderPage() {
  render(
    <NavigationProvider initialRoute={{ type: 'material', workspaceId: 'workspace-calculus-ii' }}>
      <MaterialPage workspaceId="workspace-calculus-ii" />
      <RouteObserver />
    </NavigationProvider>,
  );
}

describe('MaterialPage', () => {
  it('renders typed, concept-resolved results and the 712-page book state', async () => {
    renderPage();

    expect(await screen.findByRole('heading', { name: 'Calculus, 9th edition' })).toBeVisible();
    expect(screen.getByText('712 pages · 18 chapters')).toBeVisible();
    expect(await screen.findByText('3 typed results')).toBeVisible();
    expect(screen.getByText('Section')).toBeVisible();
    expect(screen.getByText('Worked example')).toBeVisible();
    expect(screen.getByText('Exercise range')).toBeVisible();
    expect(screen.getAllByRole('button', { name: /Shell method Developing/ })).toHaveLength(3);

    const position = screen
      .getByRole('heading', { name: 'Where you are in the book' })
      .closest('section');
    expect(within(position!).getAllByRole('listitem')).toHaveLength(4);
    expect(within(position!).getByText(/never appear in recommendations/)).toBeVisible();
    expect(screen.getByText('41 highlights · 18 notes')).toBeVisible();
    expect(screen.getByText('Most marked: §7.3, §8.2, §11.4')).toBeVisible();
  });

  it('searches material and opens the resolved concept', async () => {
    renderPage();
    const search = await screen.findByRole('searchbox', { name: 'Search this material' });
    fireEvent.change(search, { target: { value: 'integration parts' } });
    expect(await screen.findByText('1 typed result')).toBeVisible();
    expect(screen.getByText('§8.1 · Integration by Parts')).toBeVisible();

    fireEvent.click(screen.getByRole('button', { name: /Integration by parts Mastered/ }));
    await waitFor(() =>
      expect(screen.getByLabelText('Current route')).toHaveTextContent('conceptView'),
    );
  });
});
