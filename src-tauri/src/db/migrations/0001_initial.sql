CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    guiding_goal_id TEXT NOT NULL REFERENCES goals(id) DEFERRABLE INITIALLY DEFERRED,
    progress REAL NOT NULL CHECK (progress >= 0.0 AND progress <= 1.0),
    last_concept_name TEXT,
    last_activity_at TEXT,
    paused INTEGER NOT NULL CHECK (paused IN (0, 1))
);

CREATE TABLE goals (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('Guiding', 'Waiting', 'Met', 'Resting')),
    inferred_deadline TEXT,
    inferred_mastery_type TEXT,
    inferred_concept_scope INTEGER CHECK (inferred_concept_scope IS NULL OR inferred_concept_scope >= 0),
    inferred_pacing TEXT,
    previous_text TEXT,
    achieved_summary TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX one_guiding_goal_per_workspace
ON goals(workspace_id)
WHERE state = 'Guiding';

CREATE TABLE goal_tools (
    goal_id TEXT NOT NULL REFERENCES goals(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    tool TEXT NOT NULL,
    PRIMARY KEY (goal_id, position),
    UNIQUE (goal_id, tool)
);

CREATE TABLE workspace_offline_availability (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN (
        'textbookAndLectureNotes',
        'problemBanks',
        'visualAssetsAndModuleData',
        'courseVideos'
    )),
    enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
    partial_available_count INTEGER,
    partial_total_count INTEGER,
    partial_limit_reason TEXT,
    PRIMARY KEY (workspace_id, kind),
    CHECK (
        (partial_available_count IS NULL AND partial_total_count IS NULL AND partial_limit_reason IS NULL)
        OR
        (
            partial_available_count IS NOT NULL
            AND partial_total_count IS NOT NULL
            AND partial_limit_reason IS NOT NULL
            AND partial_available_count >= 0
            AND partial_total_count >= partial_available_count
        )
    )
);

CREATE TABLE workspace_activity_events (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    occurred_at TEXT NOT NULL,
    summary TEXT NOT NULL
);

CREATE INDEX workspace_activity_by_date
ON workspace_activity_events(workspace_id, occurred_at);

CREATE TABLE concepts (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    chapter TEXT NOT NULL,
    mastery_state TEXT NOT NULL CHECK (mastery_state IN ('New', 'Developing', 'Familiar', 'Strong', 'Mastered')),
    was_mastery_state TEXT CHECK (was_mastery_state IS NULL OR was_mastery_state IN ('New', 'Developing', 'Familiar', 'Strong', 'Mastered')),
    decayed_at TEXT,
    meaning TEXT NOT NULL,
    due_for_review_in_days INTEGER CHECK (due_for_review_in_days IS NULL OR due_for_review_in_days >= 0),
    on_exam INTEGER NOT NULL CHECK (on_exam IN (0, 1)),
    display_formula TEXT,
    explanation TEXT,
    learner_heuristic TEXT,
    heuristic_evidence TEXT,
    last_activity_at TEXT,
    notes_count INTEGER NOT NULL DEFAULT 0 CHECK (notes_count >= 0),
    CHECK ((was_mastery_state IS NULL) = (decayed_at IS NULL))
);

CREATE INDEX concepts_by_workspace_chapter
ON concepts(workspace_id, chapter);

CREATE TABLE concept_edges (
    source_concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
    edge_kind TEXT NOT NULL CHECK (edge_kind IN ('blocks', 'prerequisite', 'related', 'leadsTo')),
    position INTEGER NOT NULL CHECK (position >= 0),
    target_concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
    PRIMARY KEY (source_concept_id, edge_kind, position),
    UNIQUE (source_concept_id, edge_kind, target_concept_id),
    CHECK (source_concept_id <> target_concept_id)
);

CREATE INDEX concept_edges_by_target
ON concept_edges(target_concept_id, edge_kind);

CREATE TABLE concept_where_it_shows_up (
    concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    description TEXT NOT NULL,
    PRIMARY KEY (concept_id, position)
);

CREATE TABLE concept_diagnostics (
    id TEXT PRIMARY KEY,
    concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
    expression TEXT NOT NULL,
    diagnostic_type TEXT NOT NULL CHECK (diagnostic_type IN ('mistake', 'positive', 'neutral')),
    note TEXT NOT NULL,
    occurred_at TEXT NOT NULL
);

CREATE INDEX concept_diagnostics_by_date
ON concept_diagnostics(concept_id, occurred_at);

CREATE TABLE modules (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    icon TEXT NOT NULL,
    trust TEXT CHECK (trust IS NULL OR trust IN ('verified', 'community', 'experimental')),
    trust_detail TEXT,
    last_updated_label TEXT,
    learner_count_label TEXT,
    developer TEXT NOT NULL,
    price TEXT,
    description TEXT NOT NULL,
    learning_value_detail TEXT,
    context_seen TEXT NOT NULL,
    offline_status TEXT NOT NULL CHECK (offline_status IN ('Works offline', 'Online enhanced', 'Internet required')),
    enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    visibility TEXT NOT NULL CHECK (visibility IN ('workspace', 'contextual', 'off'))
);

CREATE TABLE module_supported_concepts (
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    concept_name TEXT NOT NULL,
    PRIMARY KEY (module_id, position),
    UNIQUE (module_id, concept_name)
);

CREATE TABLE module_dependencies (
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    works_with_module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    PRIMARY KEY (module_id, position),
    UNIQUE (module_id, works_with_module_id),
    CHECK (module_id <> works_with_module_id)
);

CREATE TABLE module_suitability (
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    description TEXT NOT NULL,
    PRIMARY KEY (module_id, position)
);

CREATE TABLE module_privacy_notes (
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    sentence TEXT NOT NULL,
    PRIMARY KEY (module_id, position)
);

CREATE TABLE workspace_modules (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    module_id TEXT NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    visibility TEXT NOT NULL CHECK (visibility IN ('workspace', 'contextual', 'off')),
    PRIMARY KEY (workspace_id, module_id)
);

CREATE TABLE workspace_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    tool_count INTEGER NOT NULL CHECK (tool_count >= 0)
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    concept_id TEXT NOT NULL REFERENCES concepts(id),
    status TEXT NOT NULL CHECK (status IN ('active', 'paused', 'completed')),
    intent_activity TEXT NOT NULL,
    intent_detail TEXT,
    intent_target_minutes INTEGER CHECK (intent_target_minutes IS NULL OR intent_target_minutes >= 0),
    resume_summary TEXT NOT NULL,
    thumbnail_url TEXT,
    elapsed_minutes INTEGER NOT NULL CHECK (elapsed_minutes >= 0),
    problem_index INTEGER CHECK (problem_index IS NULL OR problem_index >= 0),
    problem_count INTEGER CHECK (problem_count IS NULL OR problem_count >= 0),
    open_question TEXT,
    started_at TEXT NOT NULL,
    paused_at TEXT,
    CHECK (problem_index IS NULL OR problem_count IS NULL OR problem_index <= problem_count)
);

CREATE INDEX sessions_by_workspace_status
ON sessions(workspace_id, status);

CREATE TABLE tutor_exchanges (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    pinned_to_visualization INTEGER NOT NULL CHECK (pinned_to_visualization IN (0, 1)),
    UNIQUE (session_id, position)
);

CREATE TABLE session_settled_conclusions (
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0 AND position < 2),
    conclusion TEXT NOT NULL,
    PRIMARY KEY (session_id, position)
);

CREATE TABLE materials (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL UNIQUE REFERENCES workspaces(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    edition TEXT NOT NULL,
    total_pages INTEGER NOT NULL CHECK (total_pages > 0),
    total_chapters INTEGER NOT NULL CHECK (total_chapters > 0),
    highlights_count INTEGER NOT NULL DEFAULT 0 CHECK (highlights_count >= 0),
    notes_count INTEGER NOT NULL DEFAULT 0 CHECK (notes_count >= 0)
);

CREATE TABLE material_chapter_segments (
    material_id TEXT NOT NULL REFERENCES materials(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0 AND position < 4),
    label TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('read', 'inProgress', 'next', 'outOfSyllabus')),
    detail TEXT,
    PRIMARY KEY (material_id, position)
);

CREATE TABLE material_most_marked_sections (
    material_id TEXT NOT NULL REFERENCES materials(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    section TEXT NOT NULL,
    PRIMARY KEY (material_id, position),
    UNIQUE (material_id, section)
);

CREATE TABLE material_results (
    id TEXT PRIMARY KEY,
    material_id TEXT NOT NULL REFERENCES materials(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('section', 'workedExample', 'exerciseRange')),
    page INTEGER NOT NULL CHECK (page > 0),
    title TEXT NOT NULL,
    reason TEXT NOT NULL,
    concept_id TEXT NOT NULL REFERENCES concepts(id),
    in_syllabus INTEGER NOT NULL CHECK (in_syllabus IN (0, 1)),
    highlighted_at TEXT,
    exercise_total INTEGER CHECK (exercise_total IS NULL OR exercise_total >= 0),
    exercise_attempted INTEGER CHECK (exercise_attempted IS NULL OR exercise_attempted >= 0),
    CHECK (exercise_attempted IS NULL OR exercise_total IS NOT NULL),
    CHECK (exercise_attempted IS NULL OR exercise_attempted <= exercise_total)
);

CREATE INDEX material_results_search_scope
ON material_results(material_id, in_syllabus, kind);

CREATE TABLE notes (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    concept_id TEXT NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX notes_by_workspace_updated
ON notes(workspace_id, updated_at DESC);

CREATE TRIGGER notes_count_after_insert
AFTER INSERT ON notes
BEGIN
    UPDATE concepts
    SET notes_count = notes_count + 1
    WHERE id = NEW.concept_id;
END;

CREATE TRIGGER notes_count_after_delete
AFTER DELETE ON notes
BEGIN
    UPDATE concepts
    SET notes_count = MAX(notes_count - 1, 0)
    WHERE id = OLD.concept_id;
END;

CREATE TRIGGER notes_count_after_concept_change
AFTER UPDATE OF concept_id ON notes
WHEN OLD.concept_id <> NEW.concept_id
BEGIN
    UPDATE concepts
    SET notes_count = MAX(notes_count - 1, 0)
    WHERE id = OLD.concept_id;
    UPDATE concepts
    SET notes_count = notes_count + 1
    WHERE id = NEW.concept_id;
END;
