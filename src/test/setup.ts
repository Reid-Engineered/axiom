import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { clearMocks, mockIPC } from '@tauri-apps/api/mocks';
import { afterEach, beforeEach } from 'vitest';

import { handleMockInvoke, resetMockBackend } from './mockBackend';

beforeEach(() => {
  resetMockBackend();
  mockIPC(handleMockInvoke);
});

afterEach(() => {
  cleanup();
  clearMocks();
});
