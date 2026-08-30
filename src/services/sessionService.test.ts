import { mockIPC } from '@tauri-apps/api/mocks';
import { describe, expect, it } from 'vitest';

import { getActiveSessionByWorkspace } from './sessionService';

describe('sessionService IPC boundary', () => {
  it('preserves the undefined contract when Rust returns a null Option', async () => {
    mockIPC((command, payload) => {
      expect(command).toBe('getActiveSessionByWorkspace');
      expect(payload).toEqual({ workspaceId: 'workspace-empty' });
      return null;
    });

    await expect(getActiveSessionByWorkspace('workspace-empty')).resolves.toBeUndefined();
  });
});
