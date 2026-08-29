import { useState } from 'react';

import { Inspector } from '../components/overlays/Inspector';
import { Button } from '../components/primitives/Button';
import { Toggle } from '../components/primitives/Toggle';
import { useConcept } from '../hooks/useConcepts';
import { useNavigation } from '../hooks/useNavigation';
import { useSession } from '../hooks/useSessions';
import { FullVisualizationShell } from '../layouts/FullVisualizationShell';
import type { ShellPrimitive, VisualizationScene } from '../types';
import { shellMethodScene } from './fullVisualizationScene';
import styles from './FullVisualizationPage.module.css';

export interface FullVisualizationPageProps {
  sessionId: string;
}

/** Full-bleed visualization detour within an active study session. */
export function FullVisualizationPage({ sessionId }: FullVisualizationPageProps) {
  const { session, loading, error } = useSession(sessionId);
  const { concept } = useConcept(session?.conceptId ?? '');
  const { navigate } = useNavigation();
  const [selectedShellId, setSelectedShellId] = useState<string | null>(
    shellMethodScene.shells[0].id,
  );
  const [shellsVisible, setShellsVisible] = useState(true);
  const selectedShell = shellMethodScene.shells.find((shell) => shell.id === selectedShellId);

  if (loading || !session)
    return (
      <div className={styles.state} role="status">
        {error ? error.message : 'Opening visualization…'}
      </div>
    );

  return (
    <FullVisualizationShell
      header={
        <>
          <div className={styles.headerContext}>
            <Button
              variant="tertiary"
              size="sm"
              onClick={() => navigate({ type: 'studySession', sessionId })}
            >
              ‹ Session
            </Button>
            <strong>{shellMethodScene.name}</strong>
            <span>{concept?.name}</span>
          </div>
          <div className={styles.headerActions}>
            <Button variant="tertiary" size="sm">
              Save to notes
            </Button>
            <Button variant="tertiary" size="sm">
              Share
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => setSelectedShellId(shellMethodScene.shells[0].id)}
            >
              Inspector
            </Button>
          </div>
        </>
      }
    >
      <VisualizationStage
        scene={shellMethodScene}
        selectedShell={selectedShell}
        shellsVisible={shellsVisible}
        onShellsVisibleChange={setShellsVisible}
        onCloseInspector={() => setSelectedShellId(null)}
      />
    </FullVisualizationShell>
  );
}

function VisualizationStage({
  scene,
  selectedShell,
  shellsVisible,
  onShellsVisibleChange,
  onCloseInspector,
}: {
  scene: VisualizationScene;
  selectedShell?: ShellPrimitive;
  shellsVisible: boolean;
  onShellsVisibleChange: (visible: boolean) => void;
  onCloseInspector: () => void;
}) {
  const region = scene.regions[0];
  const functionPrimitive = scene.functions.find((item) => item.id === region.functionId);
  const axis = scene.axes[0];
  return (
    <section className={styles.stage} aria-label={scene.name}>
      <p className={styles.scenePlaceholder}>
        3D scene — composed primitives
        <br />
        {[
          scene.coordinateSystem.kind,
          functionPrimitive?.kind,
          region.kind,
          axis.kind,
          scene.revolutions[0].kind,
          scene.shells[0].kind,
          scene.annotations[0].kind,
        ]
          .filter(Boolean)
          .join(' · ')}
      </p>
      <BoundsPanel
        lower={region.lowerBound}
        upper={region.upperBound}
        shellCount={24}
        shellsVisible={shellsVisible}
        onShellsVisibleChange={onShellsVisibleChange}
      />
      <div className={styles.sceneToolbar} aria-label="Visualization tools">
        <Button variant="dark">Rotate</Button>
        <Button variant="tertiary">Slice</Button>
        <Button variant="tertiary">Revolve</Button>
        <Button variant="tertiary">Cross-section</Button>
        <Button variant="tertiary">More…</Button>
      </div>
      <div className={styles.zoomControls}>
        <Button variant="secondary" aria-label="Zoom in">
          +
        </Button>
        <Button variant="secondary" aria-label="Zoom out">
          −
        </Button>
        <Button variant="secondary" aria-label="Recentre">
          ⌾
        </Button>
      </div>
      <Inspector
        open={Boolean(selectedShell)}
        onClose={onCloseInspector}
        title="Selected shell"
        className={styles.selectedInspector}
      >
        {selectedShell ? <SelectedShell shell={selectedShell} /> : null}
      </Inspector>
    </section>
  );
}

function BoundsPanel({
  lower,
  upper,
  shellCount,
  shellsVisible,
  onShellsVisibleChange,
}: {
  lower: number;
  upper: number;
  shellCount: number;
  shellsVisible: boolean;
  onShellsVisibleChange: (visible: boolean) => void;
}) {
  const [lowerVal, setLowerVal] = useState(lower);
  const [upperVal, setUpperVal] = useState(upper);

  return (
    <section className={styles.bounds} aria-labelledby="bounds-heading">
      <h2 id="bounds-heading" className={styles.boundsTitle}>
        Bounds
      </h2>
      <label className={styles.boundsRow}>
        <span className={styles.boundsVar}>a</span>
        <input
          type="range"
          min={0}
          max={upperVal}
          step={0.1}
          value={lowerVal}
          onChange={(e) => setLowerVal(Number(e.target.value))}
          className={styles.boundsSlider}
        />
        <output className={styles.boundsOutput}>{lowerVal}</output>
      </label>
      <label className={styles.boundsRow}>
        <span className={styles.boundsVar}>b</span>
        <input
          type="range"
          min={lowerVal}
          max={4}
          step={0.1}
          value={upperVal}
          onChange={(e) => setUpperVal(Number(e.target.value))}
          className={styles.boundsSlider}
        />
        <output className={styles.boundsOutput}>{upperVal}</output>
      </label>
      <div className={styles.shellControl}>
        <span>Shells</span>
        <output>{shellCount}</output>
        <Toggle checked={shellsVisible} onChange={onShellsVisibleChange} aria-label="Show shells" />
      </div>
      <button type="button" className={styles.advancedButton}>
        Advanced…
      </button>
    </section>
  );
}

function SelectedShell({ shell }: { shell: ShellPrimitive }) {
  return (
    <div className={styles.shellDetails}>
      <dl className={styles.shellDl}>
        <div className={styles.shellDlRow}>
          <dt>radius</dt>
          <dd>x = {shell.radius}</dd>
        </div>
        <div className={styles.shellDlRow}>
          <dt>height</dt>
          <dd>x² − 1 = {shell.height}</dd>
        </div>
        <div className={styles.shellDlRow}>
          <dt>volume</dt>
          <dd>2πrh Δx ≈ {shell.approximateVolume}</dd>
        </div>
      </dl>
      <p className={styles.shellSummary}>
        This shell contributes about 6% of the total volume. Shells near x = 3 dominate.
      </p>
      <div className={styles.shellActions}>
        <Button variant="tertiary" size="sm">
          Ask the tutor
        </Button>
        <Button variant="tertiary" size="sm">
          Pin to notes
        </Button>
      </div>
    </div>
  );
}
