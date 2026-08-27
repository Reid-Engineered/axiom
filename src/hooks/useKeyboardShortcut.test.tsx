import { renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { useKeyboardShortcut } from './useKeyboardShortcut';

describe('useKeyboardShortcut', () => {
  it('handles primary-modifier shortcuts on macOS and other platforms', () => {
    const callback = vi.fn();
    renderHook(() => useKeyboardShortcut('k', callback));

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', metaKey: true }));
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'K', ctrlKey: true }));
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k' }));

    expect(callback).toHaveBeenCalledTimes(2);
  });
});
