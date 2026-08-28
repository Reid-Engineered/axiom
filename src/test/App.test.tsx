import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import App from '../App';

function enterSampleWorkspace() {
  fireEvent.click(screen.getByRole('button', { name: 'Explore a sample workspace' }));
}

describe('development gallery route', () => {
  beforeEach(() => {
    window.location.hash = '';
  });

  it('does not show the development gallery by default', () => {
    render(<App />);

    expect(screen.queryByRole('heading', { name: 'Design System Primitives' })).toBeNull();
  });

  it('shows the primitive gallery at its development hash', () => {
    window.location.hash = '#/dev/gallery';
    render(<App />);

    expect(screen.getByRole('heading', { name: 'Design System Primitives' })).toBeVisible();
  });
});

describe('Stage 3 navigation', () => {
  beforeEach(() => {
    window.location.hash = '';
  });

  it('navigates through the permanent workspace areas', () => {
    const { container } = render(<App />);

    enterSampleWorkspace();
    expect(container.querySelector('[data-route="home"]')).not.toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Calculus II' }));
    fireEvent.click(screen.getByRole('button', { name: 'Concepts' }));
    expect(container.querySelector('[data-route="conceptsList"]')).not.toBeNull();
    expect(screen.getByRole('button', { name: 'Concepts' })).toHaveAttribute(
      'aria-current',
      'page',
    );
    fireEvent.click(screen.getByRole('button', { name: 'Material' }));
    expect(container.querySelector('[data-route="material"]')).not.toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Tools' }));
    expect(container.querySelector('[data-route="workspaceTools"]')).not.toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Marketplace' }));
    expect(container.querySelector('[data-route="marketplace"]')).not.toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Home' }));
    expect(container.querySelector('[data-route="home"]')).not.toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Calculus II' }));
    expect(container.querySelector('[data-route="workspaceOverview"]')).not.toBeNull();
    fireEvent.click(screen.getByRole('button', { name: '+ New Workspace' }));
    expect(container.querySelector('[data-route="createWorkspace"]')).not.toBeNull();
  });

  it('opens the command palette from the advertised shortcut', () => {
    render(<App />);

    fireEvent.keyDown(window, { key: 'k', metaKey: true });
    expect(screen.getByRole('dialog', { name: 'Command palette' })).toBeVisible();
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(screen.queryByRole('dialog', { name: 'Command palette' })).toBeNull();
  });

  it.each([
    ['First launch', 'firstLaunch'],
    ['Study session', 'studySession'],
    ['Concept view', 'conceptView'],
    ['Module detail', 'moduleDetail'],
  ])('reaches the %s stub without adding permanent navigation', (label, route) => {
    const { container } = render(<App />);

    enterSampleWorkspace();
    fireEvent.click(screen.getByText('Page stubs'));
    fireEvent.click(screen.getByRole('button', { name: label }));
    expect(container.querySelector(`[data-route="${route}"]`)).not.toBeNull();
  });

  it('renders full visualization with a drag strip and without a sidebar', () => {
    const { container } = render(<App />);

    enterSampleWorkspace();
    fireEvent.click(screen.getByText('Page stubs'));
    fireEvent.click(screen.getByRole('button', { name: 'Full visualization' }));

    expect(container.querySelector('[data-route="fullVisualization"]')).not.toBeNull();
    expect(container.querySelector('[data-tauri-drag-region]')).not.toBeNull();
    expect(container.querySelector('aside')).toBeNull();
  });

  it('reaches the goal-editing overlay from the development stub menu', () => {
    const { container } = render(<App />);

    enterSampleWorkspace();
    fireEvent.click(screen.getByText('Page stubs'));
    fireEvent.click(screen.getByRole('button', { name: 'Goal editing' }));
    expect(container.querySelector('[data-overlay="goalEditing"]')).not.toBeNull();
  });

  it('starts without a sidebar and continues to workspace setup', () => {
    const { container } = render(<App />);

    expect(screen.getByRole('heading', { name: 'What are you learning?' })).toBeVisible();
    expect(container.querySelector('aside')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    expect(container.querySelector('[data-route="createWorkspace"]')).not.toBeNull();
  });

  it('completes the Stage 5 launch-to-populated-overview path', async () => {
    const { container } = render(<App />);

    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    fireEvent.click(screen.getByRole('button', { name: 'Create Workspace' }));
    await waitFor(() => expect(container.querySelector('[data-route="home"]')).not.toBeNull());

    fireEvent.click(screen.getAllByRole('button', { name: 'Calculus II' })[0]);
    await waitFor(() => expect(container.querySelector('[data-route="workspaceOverview"]')).not.toBeNull());
    expect(screen.getByText('Concepts in play')).toBeVisible();
    expect(screen.getByRole('button', { name: /Shell method/ })).toBeVisible();
  });
});
