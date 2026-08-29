import { useEffect, useRef, type ReactNode } from 'react';

import { useResizablePanes } from '../hooks/useResizablePanes';
import styles from './SessionShell.module.css';

/**
 * Session toolbar (44px) plus a resizable pane grid: visualization (upper, `flex: 1.35`),
 * problem (lower-left, `flex: 1.55`), tutor (lower-right). Sidebar stays visible — a
 * session is a mode of the workspace, not a separate window (screen 5).
 */
export interface SessionShellProps {
  toolbar: ReactNode;
  visualization: ReactNode;
  problem: ReactNode;
  tutor: ReactNode;
  className?: string;
}

export function SessionShell({
  toolbar,
  visualization,
  problem,
  tutor,
  className = '',
}: SessionShellProps) {
  const vertical = useResizablePanes({ initialSizes: [1.35, 1.55], minSize: 0.25 });
  const horizontal = useResizablePanes({ initialSizes: [1.55, 1], minSize: 0.25 });
  const bodyRef = useRef<HTMLDivElement>(null);
  const lowerRowRef = useRef<HTMLDivElement>(null);
  const drag = useRef<{ axis: 'vertical' | 'horizontal'; lastPosition: number } | null>(null);

  useEffect(() => {
    const move = (event: PointerEvent) => {
      if (!drag.current) return;
      const verticalDrag = drag.current.axis === 'vertical';
      const position = verticalDrag ? event.clientY : event.clientX;
      const container = verticalDrag ? bodyRef.current : lowerRowRef.current;
      const dimension = verticalDrag ? container?.clientHeight : container?.clientWidth;
      if (!dimension) return;
      const delta = (position - drag.current.lastPosition) / dimension;
      drag.current.lastPosition = position;
      (verticalDrag ? vertical : horizontal).resize(0, delta);
    };
    const stop = () => {
      drag.current = null;
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', stop);
    return () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', stop);
    };
  }, [horizontal, vertical]);

  return (
    <div className={`${styles.root} ${className}`}>
      <header className={styles.toolbar}>{toolbar}</header>
      <div className={styles.body} ref={bodyRef}>
        <div className={styles.visualization} style={{ flex: vertical.sizes[0] }}>
          {visualization}
        </div>
        <div
          className={styles.verticalDivider}
          role="separator"
          aria-orientation="horizontal"
          aria-label="Resize visualization"
          tabIndex={0}
          onPointerDown={(event) => {
            drag.current = { axis: 'vertical', lastPosition: event.clientY };
          }}
          onKeyDown={(event) => {
            if (event.key === 'ArrowUp') vertical.resize(0, -0.02);
            if (event.key === 'ArrowDown') vertical.resize(0, 0.02);
          }}
        />
        <div className={styles.lowerRow} ref={lowerRowRef} style={{ flex: vertical.sizes[1] }}>
          <div className={styles.problem} style={{ flex: horizontal.sizes[0] }}>
            {problem}
          </div>
          <div
            className={styles.horizontalDivider}
            role="separator"
            aria-orientation="vertical"
            aria-label="Resize tutor"
            tabIndex={0}
            onPointerDown={(event) => {
              drag.current = { axis: 'horizontal', lastPosition: event.clientX };
            }}
            onKeyDown={(event) => {
              if (event.key === 'ArrowLeft') horizontal.resize(0, -0.02);
              if (event.key === 'ArrowRight') horizontal.resize(0, 0.02);
            }}
          />
          <aside className={styles.tutor} style={{ flex: horizontal.sizes[1] }}>
            {tutor}
          </aside>
        </div>
      </div>
    </div>
  );
}
