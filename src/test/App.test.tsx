import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import App from '../App';

describe('development gallery route', () => {
  beforeEach(() => {
    window.location.hash = '';
  });

  it('keeps the app shell empty by default', () => {
    render(<App />);

    expect(screen.queryByRole('heading', { name: 'Design System Primitives' })).toBeNull();
  });

  it('shows the primitive gallery at its development hash', () => {
    window.location.hash = '#/dev/gallery';
    render(<App />);

    expect(screen.getByRole('heading', { name: 'Design System Primitives' })).toBeVisible();
  });
});
