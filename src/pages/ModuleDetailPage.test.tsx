import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { NavigationProvider } from '../hooks/NavigationProvider';
import { ModuleDetailPage } from './ModuleDetailPage';

function renderPage() {
  render(
    <NavigationProvider
      initialRoute={{
        type: 'moduleDetail',
        moduleId: 'module-15',
        forWorkspaceId: 'workspace-calculus-ii',
      }}
    >
      <ModuleDetailPage moduleId="module-15" forWorkspaceId="workspace-calculus-ii" />
    </NavigationProvider>,
  );
}

describe('ModuleDetailPage', () => {
  it('renders verified learning value, preview, concepts, capabilities, and metadata', async () => {
    renderPage();

    expect(await screen.findByRole('heading', { name: 'Proof Assistant' })).toBeVisible();
    expect(screen.getByText('Axiom Verified')).toBeVisible();
    expect(screen.getByText('Axiom Labs · Free · Updated last week')).toBeVisible();
    expect(screen.getByRole('button', { name: 'Install to Calculus II' })).toBeVisible();
    expect(
      screen.getByText(/Every construction uses verified mathematical primitives/),
    ).toBeVisible();
    expect(screen.getAllByText(/Preview [1-4]/)).toHaveLength(4);

    const concepts = screen.getByLabelText('Supported concepts');
    expect(
      within(concepts).getAllByText(
        /Solids of revolution|Riemann sums|Tangents and secants|Parametric curves|Vector fields/,
      ),
    ).toHaveLength(5);
    expect(screen.getByText('Your notes — off by default')).toBeVisible();
    expect(screen.getByText('Nothing leaves your device')).toBeVisible();
    expect(screen.getByText('Socratic Tutor')).toBeVisible();
    expect(screen.getByText('4.8k learners')).toBeVisible();
    expect(screen.getByText('Works offline')).toBeVisible();
  });

  it('installs only into the supplied workspace', async () => {
    renderPage();
    fireEvent.click(await screen.findByRole('button', { name: 'Install to Calculus II' }));
    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Installed in Calculus II' })).toBeDisabled(),
    );
  });
});
