import { useCallback, useState } from 'react';

export interface ResizablePanesOptions {
  initialSizes: number[];
  minSize?: number;
}

/** Owns normalized adjacent-pane proportions for pointer-driven resize controls. */
export function useResizablePanes({ initialSizes, minSize = 0.1 }: ResizablePanesOptions) {
  const normalize = useCallback((sizes: number[]) => {
    const total = sizes.reduce((sum, size) => sum + size, 0);
    return sizes.map((size) => size / total);
  }, []);
  const [sizes, setSizes] = useState(() => normalize(initialSizes));

  const resize = useCallback((dividerIndex: number, delta: number) => {
    setSizes((current) => {
      if (dividerIndex < 0 || dividerIndex >= current.length - 1) return current;
      const pairTotal = current[dividerIndex] + current[dividerIndex + 1];
      const nextLeft = Math.min(
        pairTotal - minSize,
        Math.max(minSize, current[dividerIndex] + delta),
      );
      const next = [...current];
      next[dividerIndex] = nextLeft;
      next[dividerIndex + 1] = pairTotal - nextLeft;
      return next;
    });
  }, [minSize]);

  const reset = useCallback(() => setSizes(normalize(initialSizes)), [initialSizes, normalize]);

  return { sizes, resize, reset };
}
