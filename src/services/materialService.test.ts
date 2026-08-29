import { describe, expect, it } from 'vitest';

import { searchMaterial } from './materialService';

describe('materialService', () => {
  it.each(['', 'comparison tests', 'series convergence', 'outside syllabus'])(
    'never returns out-of-syllabus material for query %j',
    async (query) => {
      const results = await searchMaterial('workspace-calculus-ii', query);

      expect(results).not.toContainEqual(
        expect.objectContaining({ id: 'material-result-series-section' }),
      );
      expect(results.every((result) => result.inSyllabus)).toBe(true);
    },
  );
});
