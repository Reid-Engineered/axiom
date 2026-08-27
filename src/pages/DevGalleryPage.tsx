import { useState } from "react";
import {
  Button,
  Chip,
  Toggle,
  ProgressBar,
  SegmentedControl,
  EyebrowLabel,
  Placeholder,
  Mastery,
  ChapterStateProfile,
  TrustBadge,
  OfflineChip,
  DiagnosticDot,
} from "../components";
import type { MasteryState } from "../types";
import styles from "./DevGalleryPage.module.css";

/**
 * Developer visual gallery for side-by-side inspection of all design system primitives.
 */
export function DevGalleryPage() {
  const [toggleState, setToggleState] = useState(true);
  const [segmentedVal, setSegmentedVal] = useState("practice");

  const segmentedOptions = [
    { value: "explain", label: "Explain" },
    { value: "practice", label: "Practice" },
    { value: "reflect", label: "Reflect" },
  ];

  const masteryStates: MasteryState[] = [
    "New",
    "Developing",
    "Familiar",
    "Strong",
    "Mastered",
  ];

  return (
    <div className={styles.container}>
      <h1 className={styles.title}>Design System Primitives</h1>
      <p className={styles.subtitle}>
        Stage 1 Component Gallery — Axiom Design System Verification
      </p>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Buttons</h2>
        <div className={styles.group}>
          <span className={styles.groupLabel}>Variants</span>
          <div className={styles.row}>
            <Button variant="primary">Primary Action</Button>
            <Button variant="secondary">Secondary Action</Button>
            <Button variant="tertiary">Tertiary Action</Button>
            <Button variant="dark">Browse Modules</Button>
            <Button variant="primary" disabled>
              Disabled
            </Button>
          </div>
        </div>
        <div className={`${styles.group} ${styles.groupSeparated}`}>
          <span className={styles.groupLabel}>Sizes</span>
          <div className={styles.row}>
            <Button size="sm" variant="primary">
              Small (sm)
            </Button>
            <Button size="md" variant="primary">
              Medium (md)
            </Button>
            <Button size="lg" variant="primary">
              Large (lg)
            </Button>
          </div>
        </div>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Chips</h2>
        <div className={styles.row}>
          <Chip label="Default Chip" />
          <Chip label="Accent Chip" variant="accent" />
          <Chip label="Subtle Chip" variant="subtle" />
          <Chip label="Deadline · Dec 12" removable onRemove={() => {}} />
          <Chip label="Removable Accent" variant="accent" removable onRemove={() => {}} />
        </div>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Toggles</h2>
        <div className={styles.row}>
          <Toggle
            checked={toggleState}
            onChange={setToggleState}
            label={toggleState ? "Axiom Tutor: On" : "Axiom Tutor: Off"}
          />
          <Toggle checked={false} onChange={() => {}} label="Off Toggle" />
          <Toggle checked={true} onChange={() => {}} disabled label="Disabled On" />
        </div>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Segmented Control</h2>
        <div className={styles.row}>
          <SegmentedControl
            options={segmentedOptions}
            value={segmentedVal}
            onChange={setSegmentedVal}
          />
        </div>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Progress Bar (unlabelled)</h2>
        <div className={styles.group}>
          <span className={styles.groupLabel}>25%</span>
          <ProgressBar value={25} />
          <span className={styles.groupLabel}>60%</span>
          <ProgressBar value={60} />
          <span className={styles.groupLabel}>100%</span>
          <ProgressBar value={100} />
        </div>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Eyebrow Labels</h2>
        <div className={styles.row}>
          <EyebrowLabel>CONTINUE</EyebrowLabel>
          <EyebrowLabel>WORKSPACES</EyebrowLabel>
          <EyebrowLabel>TOOLS & MODULES</EyebrowLabel>
        </div>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Placeholder</h2>
        <div className={styles.row}>
          <Placeholder label="Visualizer Stage 3D" className={styles.placeholderExample} />
        </div>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Mastery Ring & Words</h2>
        <div className={styles.group}>
          <span className={styles.groupLabel}>Standard (md)</span>
          <div className={styles.row}>
            {masteryStates.map((st) => (
              <Mastery key={st} state={st} />
            ))}
          </div>
        </div>
        <div className={`${styles.group} ${styles.groupSeparated}`}>
          <span className={styles.groupLabel}>Small (sm, rings only)</span>
          <div className={styles.row}>
            {masteryStates.map((st) => (
              <Mastery key={st} state={st} showLabel={false} size="sm" />
            ))}
          </div>
        </div>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Chapter State Profile</h2>
        <div className={styles.row}>
          <ChapterStateProfile
            counts={{ Mastered: 2, Strong: 3, Familiar: 1, Developing: 1, New: 1 }}
          />
          <ChapterStateProfile
            counts={{ Mastered: 4 }}
          />
        </div>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Trust Badges & Offline Chips</h2>
        <div className={styles.group}>
          <span className={styles.groupLabel}>Trust Badges</span>
          <div className={styles.row}>
            <TrustBadge type="verified" />
            <TrustBadge type="community" detail="4.8k learners" />
            <TrustBadge type="experimental" />
          </div>
        </div>
        <div className={`${styles.group} ${styles.groupSeparated}`}>
          <span className={styles.groupLabel}>Offline Chips</span>
          <div className={styles.row}>
            <OfflineChip status="Works offline" />
            <OfflineChip status="Online enhanced" />
            <OfflineChip status="Internet required" />
          </div>
        </div>
        <div className={`${styles.group} ${styles.groupSeparated}`}>
          <span className={styles.groupLabel}>Diagnostic Dots</span>
          <div className={styles.row}>
            <DiagnosticDot type="mistake" tooltip="Chose u backwards" />
            <DiagnosticDot type="positive" tooltip="Correct response" />
            <DiagnosticDot type="neutral" tooltip="Note marker" />
          </div>
        </div>
      </section>
    </div>
  );
}
