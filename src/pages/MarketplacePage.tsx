import { useState, type ReactNode } from 'react';

import { OfflineChip } from '../components/badges/OfflineChip';
import { TrustBadge } from '../components/badges/TrustBadge';
import { Button } from '../components/primitives/Button';
import { Placeholder } from '../components/primitives/Placeholder';
import { SegmentedControl } from '../components/primitives/SegmentedControl';
import { useMarketplaceModules, useWorkspaceTemplates } from '../hooks/useModules';
import { useNavigation } from '../hooks/useNavigation';
import { useWorkspaces } from '../hooks/useWorkspaces';
import { AppShell } from '../layouts/AppShell';
import type { Module } from '../types';
import styles from './MarketplacePage.module.css';

export interface MarketplacePageProps {
  forWorkspaceId?: string;
  sidebar?: ReactNode;
}

/** Module marketplace, optionally personalized for the active workspace. */
const MARKETPLACE_TABS = [
  { value: 'for-you', label: 'For you' },
  { value: 'templates', label: 'Templates' },
  { value: 'categories', label: 'Categories' },
  { value: 'installed', label: 'Installed' },
];

const GRID_MODULE_NAMES = ['Proof Assistant', 'Series Intuition Pack', 'Quiet Mode'];

/** Module marketplace, optionally personalized for the active workspace. */
export function MarketplacePage({ forWorkspaceId, sidebar }: MarketplacePageProps) {
  const { modules, loading, error, installModule } = useMarketplaceModules(forWorkspaceId);
  const { templates } = useWorkspaceTemplates();
  const { workspaces } = useWorkspaces();
  const { navigate } = useNavigation();
  const [tab, setTab] = useState('for-you');
  const featured = modules.find((module) => module.name === 'Interactive Calculus Visualizer');
  const gridModules = GRID_MODULE_NAMES.map((name) =>
    modules.find((module) => module.name === name),
  ).filter((module): module is Module => Boolean(module));
  const workspaceName =
    workspaces.find((workspace) => workspace.id === forWorkspaceId)?.name ?? 'your workspace';

  const install = (moduleId: string) => {
    if (forWorkspaceId) void installModule(moduleId);
  };

  return (
    <AppShell sidebar={sidebar}>
      <div className={styles.page}>
        <div className={styles.toolbar}>
          <label>
            <span className={styles.srOnly}>Search modules and templates</span>
            <input type="search" placeholder="Search modules and templates" />
          </label>
          <SegmentedControl options={MARKETPLACE_TABS} value={tab} onChange={setTab} />
        </div>

        <main className={styles.content}>
          <header className={styles.header}>
            <h1>For your {workspaceName} workspace</h1>
            <p>Chosen from what you are studying and how you have been studying it.</p>
          </header>

          {loading ? <p>Loading modules…</p> : null}
          {error ? <p role="alert">Marketplace modules could not be loaded.</p> : null}

          {!loading && !error && tab === 'for-you' ? (
            <>
              <div className={styles.heroRow}>
                {featured ? (
                  <article className={styles.featured}>
                    <Placeholder
                      label="module preview · 3 screenshots"
                      className={styles.preview}
                    />
                    <div className={styles.featuredBody}>
                      <div className={styles.titleRow}>
                        <h2>{featured.name}</h2>
                        {featured.trust ? <TrustBadge type={featured.trust} /> : null}
                      </div>
                      <p>{featured.description}</p>
                      <div className={styles.featuredActions}>
                        <div>
                          <Button
                            disabled={!forWorkspaceId || featured.enabled}
                            onClick={() => install(featured.id)}
                          >
                            {featured.enabled ? 'Installed' : 'Install'}
                          </Button>
                          <Button
                            variant="tertiary"
                            onClick={() =>
                              navigate({
                                type: 'moduleDetail',
                                moduleId: featured.id,
                                forWorkspaceId,
                              })
                            }
                          >
                            Learn more
                          </Button>
                        </div>
                        <span>
                          {featured.developer} · {featured.price}
                        </span>
                      </div>
                    </div>
                  </article>
                ) : null}

                <section className={styles.templates} aria-labelledby="templates-heading">
                  <h2 id="templates-heading">Workspace templates</h2>
                  {templates.map((template) => (
                    <article key={template.id}>
                      <span className={styles.eyebrow}>Workspace template</span>
                      <h3>
                        {workspaceName} — {template.name}
                      </h3>
                      <p>{template.description}</p>
                      <div>
                        <Button variant="secondary">Use template</Button>
                        <span>{template.toolCount} tools</span>
                      </div>
                    </article>
                  ))}
                </section>
              </div>

              <MarketplaceModules modules={gridModules} onInstall={install} />
              <LocalModuleRow />
            </>
          ) : null}

          {tab === 'templates' ? (
            <section className={styles.templateOnly} aria-labelledby="all-templates-heading">
              <h2 id="all-templates-heading">Workspace templates</h2>
              {templates.map((template) => (
                <p key={template.id}>
                  {template.name} · {template.toolCount} tools — {template.description}
                </p>
              ))}
            </section>
          ) : null}
          {tab === 'categories' ? <Categories /> : null}
          {tab === 'installed' ? (
            <MarketplaceModules
              title="Installed modules"
              modules={modules.filter((module) => module.enabled)}
              onInstall={install}
            />
          ) : null}
        </main>
      </div>
    </AppShell>
  );
}

function MarketplaceModules({
  modules,
  onInstall,
  title = 'Modules',
  showCategories = true,
}: {
  modules: Module[];
  onInstall: (moduleId: string) => void;
  title?: string;
  showCategories?: boolean;
}) {
  return (
    <section className={styles.moduleSection} aria-labelledby={`${title}-heading`}>
      <div className={styles.moduleHeaderRow}>
        <h2 id={`${title}-heading`}>{title}</h2>
        {showCategories ? <Categories /> : null}
      </div>
      <div className={styles.moduleGrid}>
        {modules.map((module) => (
          <article key={module.id}>
            <div className={styles.moduleTitle}>
              <span className={styles.icon}>{module.icon}</span>
              <h3>{module.name}</h3>
            </div>
            {module.trust ? <TrustBadge type={module.trust} detail={module.trustDetail} /> : null}
            <p className={styles.moduleDescription}>{module.description}</p>
            {module.name === 'Quiet Mode' && module.suits?.[0] ? (
              <p className={styles.suits}>Suits: {module.suits[0]}</p>
            ) : null}
            <OfflineChip status={module.offlineStatus} />
            <div className={styles.moduleActions}>
              <Button
                variant="secondary"
                disabled={module.enabled}
                onClick={() => onInstall(module.id)}
              >
                {module.enabled ? 'Installed' : 'Install'}
              </Button>
              <span className={styles.developerName}>{module.developer}</span>
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function Categories() {
  return (
    <nav className={styles.categories} aria-label="Module categories">
      {['Mathematics', 'Science', 'Languages', 'Memory', 'Accessibility'].map((category) => (
        <button type="button" key={category}>
          {category}
        </button>
      ))}
    </nav>
  );
}

function LocalModuleRow() {
  return (
    <div className={styles.localModule}>
      <span>Building something yourself? Load a local module into a workspace to try it.</span>
      <button type="button">Load local module</button>
    </div>
  );
}
