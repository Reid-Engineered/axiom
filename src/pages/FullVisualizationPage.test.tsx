import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { useNavigation } from '../hooks/useNavigation';
import { FullVisualizationPage } from './FullVisualizationPage';
import { shellMethodScene } from './fullVisualizationScene';

function RouteObserver() {
  const { route } = useNavigation();
  return <output aria-label="Current route">{route.type}</output>;
}

function renderPage() {
  render(
    <NavigationProvider
      initialRoute={{ type: 'fullVisualization', sessionId: 'session-shell-method' }}
    >
      <FullVisualizationPage sessionId="session-shell-method" />
      <RouteObserver />
    </NavigationProvider>,
  );
}

describe('FullVisualizationPage', () => {
  it('defines and renders the complete verified-primitive scene shape', async () => {
    expect(shellMethodScene.coordinateSystem.kind).toBe('coordinate-system');
    expect(shellMethodScene.functions[0].kind).toBe('function');
    expect(shellMethodScene.regions[0].kind).toBe('region');
    expect(shellMethodScene.axes[0].kind).toBe('axis');
    expect(shellMethodScene.revolutions[0].kind).toBe('revolution');
    expect(shellMethodScene.shells[0].kind).toBe('shell');
    expect(shellMethodScene.annotations[0].kind).toBe('annotation');

    renderPage();
    expect(await screen.findByLabelText('Shells about x = 0')).toBeVisible();
    expect(
      screen.getByText(
        /coordinate-system · function · region · axis · revolution · shell · annotation/,
      ),
    ).toBeVisible();
    expect(screen.getAllByRole('slider')).toHaveLength(2);
    expect(screen.getByRole('switch', { name: 'Show shells' })).toBeChecked();
    expect(screen.getByRole('complementary', { name: 'Selected shell' })).toBeVisible();
    expect(screen.getByText('2πrh Δx ≈ 4.48')).toBeVisible();
  });

  it('dismisses and restores selection, toggles shells, and returns to the session', async () => {
    renderPage();
    await screen.findByLabelText('Shells about x = 0');

    fireEvent.click(screen.getByRole('button', { name: 'Close Selected shell' }));
    expect(screen.queryByRole('complementary', { name: 'Selected shell' })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Inspector' }));
    expect(screen.getByRole('complementary', { name: 'Selected shell' })).toBeVisible();

    fireEvent.click(screen.getByRole('switch', { name: 'Show shells' }));
    expect(screen.getByRole('switch', { name: 'Show shells' })).not.toBeChecked();
    fireEvent.click(screen.getByRole('button', { name: '‹ Session' }));
    await waitFor(() =>
      expect(screen.getByLabelText('Current route')).toHaveTextContent('studySession'),
    );
  });
});
