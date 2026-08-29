import type { Module, WorkspaceTemplate } from '../../types';

const moduleNames = [
  'Socratic Tutor',
  'Function Visualizer',
  'Adaptive Practice',
  'Symbolic Algebra',
  'Concept Notes',
  'Review Planner',
  'Graph Explorer',
  'Worked Examples',
  'Proof Coach',
  'Formula Interpreter',
  'Mistake Patterns',
  'Exam Rehearsal',
  'Reading Companion',
  '3D Learning Canvas',
  'Problem Generator',
  'Study Reflection',
  'Prerequisite Mapper',
  'Course Video Guide',
  'Notation Translator',
  'Focus Timer',
];

const marketplaceOverrides: Partial<Record<number, Partial<Module>>> = {
  13: {
    name: 'Interactive Calculus Visualizer',
    icon: 'V',
    trust: 'verified',
    developer: 'Axiom Labs',
    price: 'Free',
    description:
      'Manipulate solids, Riemann sums, and tangent constructions while the tutor explains each step.',
    trustDetail: 'Updated last week',
  },
  14: {
    name: 'Proof Assistant',
    icon: 'P',
    trust: 'verified',
    developer: 'Axiom Labs',
    price: 'Free',
    description:
      'Work through a derivation step by step; it checks each move and names the rule you used.',
    learningValueDetail:
      'Every construction uses verified mathematical primitives, so the explanation and the visual state stay aligned.',
    lastUpdatedLabel: 'Updated last week',
    learnerCountLabel: '4.8k learners',
    trustDetail: undefined,
    offlineStatus: 'Works offline',
    supportedConceptNames: [
      'Solids of revolution',
      'Riemann sums',
      'Tangents and secants',
      'Parametric curves',
      'Vector fields',
    ],
    worksWithModuleIds: ['module-1', 'module-3', 'module-5', 'module-6'],
    suits: ['Calculus I–III, Multivariable, Differential Equations', 'Learners who think visually'],
    privacyNotes: [
      'Your current problem — while the module is open',
      'Your notes — off by default',
      'Your workspace goal — used to keep examples relevant',
      'Nothing leaves your device',
    ],
  },
  15: {
    name: 'Series Intuition Pack',
    icon: 'S',
    trust: 'community',
    trustDetail: '4.8k learners',
    developer: 'M. Okonkwo',
    price: 'Free',
    description:
      'Treat convergence tests as decisions rather than a table, with animated partial sums.',
  },
  16: {
    name: 'Quiet Mode',
    icon: 'Q',
    trust: 'community',
    trustDetail: 'Accessibility',
    developer: 'P. Lindqvist',
    price: 'Free',
    description:
      'Removes timers and motion, keeps one activity on screen, and allows longer pauses before feedback.',
    suits: ['Learners who prefer reduced motion', 'Learners who need a quieter pace'],
  },
};

export const mockModules: Module[] = moduleNames.map((name, index) => {
  const visibility = index < 4 ? 'workspace' : index < 13 ? 'contextual' : 'off';
  const enabled = index < 13;

  return {
    id: `module-${index + 1}`,
    name,
    icon: name.slice(0, 1),
    ...(index < 6
      ? {}
      : { trust: index % 3 === 0 ? ('experimental' as const) : ('community' as const) }),
    ...(index >= 6
      ? { trustDetail: index % 3 === 0 ? 'Updated last week' : `${1200 + index * 317} learners` }
      : {}),
    developer: index < 6 ? 'Axiom' : `Learning Lab ${index - 5}`,
    ...(index >= 13 ? { price: index % 2 === 0 ? '$8' : 'Free' } : {}),
    description: `Supports ${name.toLowerCase()} while keeping the current concept and goal in view.`,
    contextSeen:
      index % 2 === 0
        ? 'The current concept, workspace goal, and answers from this session.'
        : 'The material you open and the concept you are studying.',
    offlineStatus:
      index % 7 === 0 ? 'Internet required' : index % 3 === 0 ? 'Online enhanced' : 'Works offline',
    supportedConceptNames: ['Shell method', 'Integration by parts', 'Taylor series'],
    worksWithModuleIds: index > 0 ? [`module-${Math.max(1, index)}`] : [],
    suits: ['Learners who prefer worked examples', 'Exam preparation'],
    privacyNotes: [
      'Your current problem — while the module is open',
      'Your notes — off by default',
    ],
    enabled,
    visibility,
    ...marketplaceOverrides[index],
  };
});

export const mockWorkspaceTemplates: WorkspaceTemplate[] = [
  {
    id: 'template-visual-learner',
    name: 'Visual Learner',
    description: 'Tutor, symbolic tools, 2D and 3D visualization, practice, and spaced review.',
    toolCount: 7,
  },
  {
    id: 'template-exam-intensive',
    name: 'Exam Intensive',
    description: 'Timed practice, weakness detection, formula review, and exam simulation.',
    toolCount: 5,
  },
];
