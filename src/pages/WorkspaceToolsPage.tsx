import { useEffect, useState, type ReactNode } from 'react';

import { SuggestionPanel } from '../components/feedback/SuggestionPanel';
import { Sheet } from '../components/overlays/Sheet';
import { Button } from '../components/primitives/Button';
import { Toggle } from '../components/primitives/Toggle';
import { useGoals } from '../hooks/useGoals';
import { useModules } from '../hooks/useModules';
import { useNavigation } from '../hooks/useNavigation';
import { useWorkspaces } from '../hooks/useWorkspaces';
import { AppShell } from '../layouts/AppShell';
import type { Module, OfflineContentKind, OfflineKindAvailability } from '../types';
import styles from './WorkspaceToolsPage.module.css';

export interface WorkspaceToolsPageProps {
  workspaceId: string;
  sidebar?: ReactNode;
}

/** Module visibility and offline controls for one workspace. */
export function WorkspaceToolsPage({ workspaceId, sidebar }: WorkspaceToolsPageProps) {
  const { modules, setEnabled } = useModules(workspaceId);
  const { goals } = useGoals(workspaceId);
  const { workspaces } = useWorkspaces();
  const { navigate } = useNavigation();
  const [offlineOpen, setOfflineOpen] = useState(false);
  const [suggestionVisible, setSuggestionVisible] = useState(true);
  const workspace = workspaces.find((candidate) => candidate.id === workspaceId);
  const groups = { workspace: modules.filter((module) => module.visibility === 'workspace'), contextual: modules.filter((module) => module.visibility === 'contextual'), off: modules.filter((module) => module.visibility === 'off') };

  return <AppShell sidebar={sidebar}><main className={styles.main}><header className={styles.header}><div><h1>Tools in this workspace</h1><p>Turn something off and it disappears from the workspace — nothing you’ve made with it is deleted.</p></div><div><Button variant="secondary" onClick={() => setOfflineOpen(true)}>Make available offline</Button><Button variant="dark" onClick={() => navigate({ type: 'marketplace', forWorkspaceId: workspaceId })}>Browse modules</Button></div></header><p className={styles.scaleSummary}>{modules.length} modules · {goals.length} goals</p><ModuleGroup title="In the workspace" modules={groups.workspace} onToggle={(module, enabled) => setEnabled(module.id, enabled)} /><ModuleGroup title="Appear when relevant" modules={groups.contextual} onToggle={(module, enabled) => setEnabled(module.id, enabled)} /><ModuleGroup title="Off in this workspace" modules={groups.off} onToggle={(module, enabled) => setEnabled(module.id, enabled)} />{suggestionVisible ? <SuggestionPanel message="Proof Coach might suit this workspace because you keep asking the tutor why a method works." acceptLabel="Take a look" onAccept={() => navigate({ type: 'moduleDetail', moduleId: 'module-9', forWorkspaceId: workspaceId })} onDismiss={() => setSuggestionVisible(false)} /> : null}</main><OfflineSheet open={offlineOpen} onClose={() => setOfflineOpen(false)} workspaceName={workspace?.name ?? 'Workspace'} availability={workspace?.offlineAvailability ?? []} /></AppShell>;
}

function ModuleGroup({ title, modules, onToggle }: { title: string; modules: Module[]; onToggle: (module: Module, enabled: boolean) => void }) {
  return <section className={styles.group} aria-labelledby={`${title.replace(/ /g, '-').toLowerCase()}-title`}><h2 id={`${title.replace(/ /g, '-').toLowerCase()}-title`}>{title} · {modules.length}</h2><div>{modules.map((module) => <article key={module.id} className={styles.moduleRow}><span className={styles.icon}>{module.icon}</span><div><h3>{module.name}</h3><p>{module.description}</p></div><Toggle checked={module.enabled} onChange={(enabled) => onToggle(module, enabled)} aria-label={`${module.name} enabled`} /></article>)}</div></section>;
}

function OfflineSheet({ open, onClose, workspaceName, availability }: { open: boolean; onClose: () => void; workspaceName: string; availability: OfflineKindAvailability[] }) {
  const [choices, setChoices] = useState(availability);
  useEffect(() => setChoices(availability), [availability]);
  const total = choices.filter((choice) => choice.enabled).reduce((sum, choice) => sum + choice.sizeBytes, 0);
  return <Sheet open={open} onClose={onClose} eyebrow={workspaceName} title="Make available offline" footer={<><Button onClick={onClose}>Download · {formatBytes(total)}</Button><Button variant="secondary" onClick={onClose}>Cancel</Button></>}><p className={styles.offlineIntro}>Study on a plane, in a library basement, anywhere. Your work syncs when you’re back.</p><div className={styles.offlineKinds}>{choices.map((choice) => <article key={choice.kind}><div><h3>{offlineLabel(choice.kind)}</h3>{choice.partial ? <p>{choice.partial.availableCount} of {choice.partial.totalCount} downloadable — {choice.partial.limitReason}</p> : null}</div><span>{formatBytes(choice.sizeBytes)}</span><Toggle checked={choice.enabled} onChange={(enabled) => setChoices((current) => current.map((item) => item.kind === choice.kind ? { ...item, enabled } : item))} aria-label={`${offlineLabel(choice.kind)} offline`} /></article>)}</div><p className={styles.degradation}>Voice tutoring requires internet. Typed tutoring keeps working offline, with slightly shorter answers.</p></Sheet>;
}

function offlineLabel(kind: OfflineContentKind) { return ({ textbookAndLectureNotes: 'Textbook & lecture notes', problemBanks: 'Problem banks', visualAssetsAndModuleData: 'Visual assets & module data', courseVideos: 'Course videos' } as const)[kind]; }
function formatBytes(bytes: number) { const gigabytes = bytes / 1_073_741_824; return gigabytes >= 1 ? `${gigabytes.toFixed(1)} GB` : `${Math.round(bytes / 1_048_576)} MB`; }
