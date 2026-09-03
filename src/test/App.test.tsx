import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import App from '../App';

/**
 * "Explore a sample workspace" appears twice once the empty sidebar is visible alongside
 * First Launch's own content (same action, two entry points) — this picks the one outside
 * the sidebar, matching a learner reading the page rather than using the persistent chrome.
 */
async function enterSampleWorkspace() {
  const sidebar = screen.queryByRole('navigation', { name: 'Primary' });
  const buttons = screen.getAllByRole('button', { name: 'Explore a sample workspace' });
  const pageButton = buttons.find((button) => !sidebar?.contains(button)) ?? buttons[0];
  fireEvent.click(pageButton);
  await waitFor(() => expect(document.querySelector('[data-route="home"]')).not.toBeNull());
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

  it('navigates through the permanent workspace areas', async () => {
    const { container } = render(<App />);

    await enterSampleWorkspace();
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

  it('opens real command palette results from the advertised shortcut', async () => {
    render(<App />);

    fireEvent.keyDown(window, { key: 'k', metaKey: true });
    const dialog = screen.getByRole('dialog', { name: 'Command palette' });
    expect(dialog).toBeVisible();
    expect(await within(dialog).findByText('Practice the Shell Method')).toBeVisible();
    expect(within(dialog).getByText('From your work')).toBeVisible();
    expect(within(dialog).getByText(/Note —/)).toBeVisible();
    expect(within(dialog).getByText('Marketplace')).toBeVisible();
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(screen.queryByRole('dialog', { name: 'Command palette' })).toBeNull();
  });

  it.each([
    ['First launch', 'firstLaunch'],
    ['Study session', 'studySession'],
    ['Concept view', 'conceptView'],
    ['Module detail', 'moduleDetail'],
  ])('reaches the %s stub without adding permanent navigation', async (label, route) => {
    const { container } = render(<App />);

    await enterSampleWorkspace();
    fireEvent.click(screen.getByText('Page stubs'));
    fireEvent.click(screen.getByRole('button', { name: label }));
    expect(container.querySelector(`[data-route="${route}"]`)).not.toBeNull();
  });

  it('renders full visualization with a drag strip and without a sidebar', async () => {
    const { container } = render(<App />);

    await enterSampleWorkspace();
    fireEvent.click(screen.getByText('Page stubs'));
    fireEvent.click(screen.getByRole('button', { name: 'Full visualization' }));

    expect(container.querySelector('[data-route="fullVisualization"]')).not.toBeNull();
    expect(container.querySelector('[data-tauri-drag-region]')).not.toBeNull();
    expect(container.querySelector('aside')).toBeNull();
  });

  it('reaches the goal-editing overlay from the development stub menu', async () => {
    const { container } = render(<App />);

    await enterSampleWorkspace();
    fireEvent.click(screen.getByText('Page stubs'));
    fireEvent.click(screen.getByRole('button', { name: 'Goal editing' }));
    expect(container.querySelector('[data-overlay="goalEditing"]')).not.toBeNull();
  });

  it('shows a visible sidebar (not absent) on first launch and continues to workspace setup', () => {
    const { container } = render(<App />);

    expect(screen.getByRole('heading', { name: 'What are you learning?' })).toBeVisible();
    expect(container.querySelector('aside')).not.toBeNull();
    expect(screen.getByRole('navigation', { name: 'Primary' })).toBeVisible();

    expect(screen.getByRole('textbox', { name: 'Subject' })).toHaveValue('');
    expect(screen.getByPlaceholderText('Calculus II')).toBeVisible();
    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    expect(container.querySelector('[data-route="createWorkspace"]')).not.toBeNull();
    expect(container.querySelector('aside')).not.toBeNull();
  });

  it('reflects a newly created workspace in the sidebar once Home is reached', async () => {
    const { container } = render(<App />);

    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    fireEvent.change(screen.getByLabelText('Subject'), {
      target: { value: 'Sidebar Regression Subject' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Create Workspace' }));
    await waitFor(() => expect(container.querySelector('[data-route="home"]')).not.toBeNull());

    const sidebar = screen.getByRole('navigation', { name: 'Primary' });
    await waitFor(() =>
      expect(
        within(sidebar).getByRole('button', { name: 'Sidebar Regression Subject' }),
      ).toBeVisible(),
    );
  });

  it('completes the Stage 5 launch-to-populated-overview path', async () => {
    const { container } = render(<App />);

    fireEvent.click(screen.getByRole('button', { name: 'Continue' }));
    fireEvent.click(screen.getByRole('button', { name: 'Create Workspace' }));
    await waitFor(() => expect(container.querySelector('[data-route="home"]')).not.toBeNull());

    fireEvent.click(screen.getAllByRole('button', { name: 'Calculus II' })[0]);
    await waitFor(() =>
      expect(container.querySelector('[data-route="workspaceOverview"]')).not.toBeNull(),
    );
    expect(screen.getByText('Concepts in play')).toBeVisible();
    expect(screen.getByRole('button', { name: /Shell method/ })).toBeVisible();
  });
});
