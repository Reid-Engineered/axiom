import { OfflineChip } from '../components/badges/OfflineChip';
import { TrustBadge } from '../components/badges/TrustBadge';
import { ConceptTag } from '../components/concept/ConceptTag';
import { Button } from '../components/primitives/Button';
import { Placeholder } from '../components/primitives/Placeholder';
import { useMarketplaceModules, useModule } from '../hooks/useModules';
import { useNavigation } from '../hooks/useNavigation';
import { useWorkspaces } from '../hooks/useWorkspaces';
import { TwoPaneLayout } from '../layouts/TwoPaneLayout';
import styles from './ModuleDetailPage.module.css';

export interface ModuleDetailPageProps {
  moduleId: string;
  forWorkspaceId?: string;
}

/** Learning value, trust, context access, and workspace-scoped actions for one module. */
export function ModuleDetailPage({ moduleId, forWorkspaceId }: ModuleDetailPageProps) {
  const { module: catalogModule, loading, error } = useModule(moduleId);
  const { modules, installModule } = useMarketplaceModules(forWorkspaceId);
  const { workspaces } = useWorkspaces();
  const { navigate } = useNavigation();
  const scopedModule = modules.find((module) => module.id === moduleId);
  const module = scopedModule ?? catalogModule;
  const workspace = workspaces.find((candidate) => candidate.id === forWorkspaceId);
  const worksWith = module?.worksWithModuleIds
    ?.map((id) => modules.find((candidate) => candidate.id === id)?.name)
    .filter((name): name is string => Boolean(name));

  const install = () => {
    if (forWorkspaceId) void installModule(moduleId);
  };

  const rail = module ? (
    <div className={styles.railContent}>
      <section>
        <h2>What it can see</h2>
        <ul className={styles.capabilities}>
          {module.privacyNotes?.map((note) => (
            <li key={note}>{note}</li>
          ))}
        </ul>
        <button type="button" className={styles.link}>
          Change what it sees
        </button>
      </section>
      <section>
        <h2>Works with</h2>
        <ul className={styles.worksWith}>
          {worksWith?.map((name) => (
            <li key={name}>
              <span className={styles.worksWithIcon} aria-hidden="true" />
              <span>{name}</span>
            </li>
          ))}
        </ul>
      </section>
      <section>
        <h2>Suits</h2>
        <p>{module.suits?.join('. ')}</p>
      </section>
      <dl className={styles.metadata}>
        <div>
          <dt>Developer</dt>
          <dd>{module.developer}</dd>
        </div>
        <div>
          <dt>Learners using it</dt>
          <dd>{module.learnerCountLabel}</dd>
        </div>
        <div>
          <dt>Offline</dt>
          <dd>
            <OfflineChip status={module.offlineStatus} />
          </dd>
        </div>
      </dl>
    </div>
  ) : null;

  return (
    <TwoPaneLayout rail={rail} className={styles.layout}>
      {loading ? <p>Loading module…</p> : null}
      {error ? <p role="alert">Module details could not be loaded.</p> : null}
      {module ? (
        <div className={styles.content}>
          <nav className={styles.breadcrumb} aria-label="Breadcrumb">
            <button
              type="button"
              className={styles.breadcrumbLink}
              onClick={() => navigate({ type: 'marketplace', forWorkspaceId })}
            >
              Marketplace
            </button>
            <span className={styles.breadcrumbSeparator}>›</span>
            <span className={styles.breadcrumbCurrent}>{module.name}</span>
          </nav>

          <header className={styles.header}>
            <span className={styles.icon}>{module.icon}</span>
            <div>
              <div className={styles.titleRow}>
                <h1>{module.name}</h1>
                {module.trust ? <TrustBadge type={module.trust} /> : null}
              </div>
              <p>
                {module.developer} · {module.price} · {module.lastUpdatedLabel}
              </p>
            </div>
          </header>

          <div className={styles.actions}>
            <Button disabled={!forWorkspaceId || module.enabled} onClick={install}>
              {module.enabled
                ? `Installed in ${workspace?.name ?? 'workspace'}`
                : `Install to ${workspace?.name ?? 'workspace'}`}
            </Button>
            <Button variant="secondary">Try it first</Button>
            <Button variant="tertiary">Add to another workspace</Button>
          </div>

          <section className={styles.preview} aria-label="Module preview">
            <Placeholder
              label="interactive preview · live, sandboxed"
              className={styles.livePreview}
            />
            <div className={styles.thumbnails}>
              {Array.from({ length: 4 }, (_, index) => (
                <Placeholder key={index} label={`Preview ${index + 1}`} />
              ))}
            </div>
          </section>

          <section className={styles.learningValue}>
            <h2>What it adds to your learning</h2>
            <p>{module.description}</p>
            {module.learningValueDetail ? <p>{module.learningValueDetail}</p> : null}
            <div className={styles.concepts} aria-label="Supported concepts">
              {module.supportedConceptNames?.map((concept) => (
                <ConceptTag key={concept} label={concept} />
              ))}
            </div>
          </section>
        </div>
      ) : null}
    </TwoPaneLayout>
  );
}
