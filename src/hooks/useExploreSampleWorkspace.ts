import { useCallback, useState } from 'react';

import { useNavigation } from './useNavigation';
import { useWorkspace } from './useWorkspace';
import { useWorkspaces } from './useWorkspaces';

/**
 * Opt-in sample-workspace import shared by every entry point that offers it (First
 * Launch's own content and the sidebar's empty-state action). Never runs automatically.
 */
export function useExploreSampleWorkspace() {
  const { navigate } = useNavigation();
  const { setActiveWorkspaceId } = useWorkspace();
  const { importSampleWorkspace } = useWorkspaces();
  const [importing, setImporting] = useState(false);
  const [error, setError] = useState<string>();

  const explore = useCallback(async () => {
    setImporting(true);
    setError(undefined);
    try {
      const workspace = await importSampleWorkspace();
      setActiveWorkspaceId(workspace.id);
      navigate({ type: 'home' });
    } catch {
      setError('The sample workspace could not be prepared. Try again.');
    } finally {
      setImporting(false);
    }
  }, [importSampleWorkspace, navigate, setActiveWorkspaceId]);

  return { explore, importing, error };
}
