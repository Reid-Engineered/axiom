import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { mockModules } from '../services/mockData/modules';
import { useMarketplaceModules, useModule, useModules } from './useModules';

describe('module domain hooks', () => {
  it('loads the real 4/9/7 grouping and toggles a module', async () => {
    const { result } = renderHook(() => useModules('workspace-calculus-ii'));
    await waitFor(() => expect(result.current.modules).toHaveLength(20));
    expect(result.current.modules.filter((module) => module.visibility === 'workspace')).toHaveLength(4);
    expect(result.current.modules.filter((module) => module.visibility === 'contextual')).toHaveLength(9);
    expect(result.current.modules.filter((module) => module.visibility === 'off')).toHaveLength(7);

    const module = mockModules[mockModules.length - 1];
    if (!module) throw new Error('Module fixture missing');
    await act(async () => {
      await result.current.setEnabled(module.id, true);
    });
    expect(result.current.modules[result.current.modules.length - 1]).toMatchObject({ enabled: true, visibility: 'contextual' });
    await act(async () => {
      await result.current.setVisibility(module.id, 'off');
    });
  });

  it('loads and installs from the real marketplace catalog', async () => {
    const module = mockModules[mockModules.length - 1];
    if (!module) throw new Error('Module fixture missing');
    const { result } = renderHook(() => useMarketplaceModules('workspace-calculus-ii'));
    await waitFor(() => expect(result.current.modules).toHaveLength(20));
    await act(async () => {
      await result.current.installModule(module.id);
    });
    expect(result.current.modules[result.current.modules.length - 1]?.enabled).toBe(true);
  });

  it('loads one module from the catalog fixture', async () => {
    const fixture = mockModules[1];
    const { result } = renderHook(() => useModule(fixture.id));
    await waitFor(() => expect(result.current.module?.id).toBe(fixture.id));
    expect(result.current.module?.description).toContain('function visualizer');
  });
});
