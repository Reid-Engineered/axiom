import type { Module } from '../../types';

const moduleNames = [
  'Socratic Tutor', 'Function Visualizer', 'Adaptive Practice', 'Symbolic Algebra',
  'Concept Notes', 'Review Planner', 'Graph Explorer', 'Worked Examples',
  'Proof Coach', 'Formula Interpreter', 'Mistake Patterns', 'Exam Rehearsal',
  'Reading Companion', '3D Learning Canvas', 'Problem Generator', 'Study Reflection',
  'Prerequisite Mapper', 'Course Video Guide', 'Notation Translator', 'Focus Timer',
];

export const mockModules: Module[] = moduleNames.map((name, index) => {
  const visibility = index < 4 ? 'workspace' : index < 13 ? 'contextual' : 'off';
  const enabled = index < 13;

  return {
    id: `module-${index + 1}`,
    name,
    icon: name.slice(0, 1),
    ...(index < 6 ? {} : { trust: index % 3 === 0 ? 'experimental' as const : 'community' as const }),
    ...(index >= 6 ? { trustDetail: index % 3 === 0 ? 'Updated last week' : `${1200 + index * 317} learners` } : {}),
    developer: index < 6 ? 'Axiom' : `Learning Lab ${index - 5}`,
    ...(index >= 13 ? { price: index % 2 === 0 ? '$8' : 'Free' } : {}),
    description: `Supports ${name.toLowerCase()} while keeping the current concept and goal in view.`,
    contextSeen: index % 2 === 0
      ? 'The current concept, workspace goal, and answers from this session.'
      : 'The material you open and the concept you are studying.',
    offlineStatus: index % 7 === 0 ? 'Internet required' : index % 3 === 0 ? 'Online enhanced' : 'Works offline',
    supportedConceptNames: ['Shell method', 'Integration by parts', 'Taylor series'],
    worksWithModuleIds: index > 0 ? [`module-${Math.max(1, index)}`] : [],
    suits: ['Learners who prefer worked examples', 'Exam preparation'],
    privacyNotes: ['Your current problem — while the module is open', 'Your notes — off by default'],
    enabled,
    visibility,
  };
});
