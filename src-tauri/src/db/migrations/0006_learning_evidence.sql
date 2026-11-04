CREATE TABLE practice_policy_snapshots (
    id TEXT NOT NULL,
    revision INTEGER NOT NULL CHECK (revision > 0),
    digest TEXT NOT NULL,
    approved_by TEXT NOT NULL,
    decision_ref TEXT NOT NULL,
    effective_at TEXT NOT NULL,
    policy_json TEXT NOT NULL,
    PRIMARY KEY (id, revision)
);

INSERT INTO practice_policy_snapshots (
    id, revision, digest, approved_by, decision_ref, effective_at, policy_json
) VALUES (
    'policy.reid_default',
    1,
    'sha256:9b5781d0a8a3e5a638f18ef40219c6a09446e0470b14eb93eb5f07c7bf01b102',
    'human-review',
    '083-learning-evidence-contracts:P1-P7:2026-11-04',
    '2026-11-04T19:30:00Z',
    '{"mastery":{"trial_weight":0.25,"partial_credit":0.5,"named_thresholds":{"Developing":0.20,"Familiar":0.45,"Strong":0.70,"Mastered":0.85},"minimum_mastered_trials":3,"objective_aggregation":"weighted-mean"},"decay":{"half_life_ms":1209600000,"projection_cadence":"session-and-day"},"readiness":{"threshold":0.60},"bloom":{"attempt_count":2,"session_count":1,"unassisted_required":true},"resolution":{"required_clean":3,"editable_min":2,"editable_max":5,"unknown_resets":false},"difficulty":{"Steady":{"hint_budget":3,"cost_per_hint":0.20,"solution_credit":0.0},"Push":{"hint_budget":1,"cost_per_hint":0.35,"solution_credit":0.0},"Exam":{"hint_budget":0,"cost_per_hint":1.0,"solution_credit":0.0}},"planning":{"default_target_ms":1500000,"default_question_limit":5,"question_limit_range":[3,8],"target_ms_range":[600000,2700000],"phase_shares":{"warmup":0.20,"stretch":0.60,"combined":0.20},"replacement_limit":1},"timing_policy_ref":"timing.foreground_5s_v1"}'
);

CREATE TABLE practice_content_snapshots (
    digest TEXT PRIMARY KEY,
    package_id TEXT NOT NULL,
    package_version TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    content_json TEXT NOT NULL,
    recorded_at TEXT NOT NULL
);

CREATE TABLE practice_observations (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    attempt_id TEXT NOT NULL,
    submission_id TEXT,
    slot_id TEXT NOT NULL,
    concept_id TEXT NOT NULL,
    objective_ids_json TEXT NOT NULL,
    family_id TEXT NOT NULL,
    seed TEXT NOT NULL,
    bloom_level TEXT,
    difficulty TEXT NOT NULL CHECK (difficulty IN ('Steady', 'Push', 'Exam')),
    outcome TEXT NOT NULL CHECK (outcome IN ('correct', 'partial', 'incorrect', 'solution-revealed', 'skipped', 'unverifiable')),
    assessable INTEGER NOT NULL CHECK (assessable IN (0, 1)),
    raw_credit REAL,
    hint_cost REAL NOT NULL DEFAULT 0.0,
    effective_credit REAL,
    hints_used INTEGER NOT NULL DEFAULT 0,
    verification_id TEXT,
    attribution_id TEXT,
    policy_id TEXT NOT NULL,
    policy_revision INTEGER NOT NULL,
    content_digest TEXT NOT NULL,
    observed_at TEXT NOT NULL,
    stamp_json TEXT NOT NULL
);

CREATE INDEX practice_observations_by_workspace_concept
ON practice_observations(workspace_id, concept_id, observed_at);

CREATE INDEX practice_observations_by_session
ON practice_observations(session_id, observed_at);

CREATE TABLE practice_verifications (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    attempt_id TEXT NOT NULL,
    submission_id TEXT,
    verdict TEXT NOT NULL CHECK (verdict IN ('correct', 'incorrect', 'partial', 'unverifiable')),
    method TEXT NOT NULL,
    checker_version TEXT NOT NULL,
    verification_json TEXT NOT NULL,
    stamp_json TEXT NOT NULL,
    recorded_at TEXT NOT NULL
);

CREATE TABLE practice_attributions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    observation_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('confirmed', 'candidate', 'unknown', 'none', 'input-error')),
    pattern_id TEXT,
    detector_id TEXT,
    attribution_json TEXT NOT NULL,
    stamp_json TEXT NOT NULL,
    recorded_at TEXT NOT NULL
);

CREATE TABLE practice_mastery_snapshots (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('concept', 'objective')),
    subject_id TEXT NOT NULL,
    observation_id TEXT NOT NULL,
    raw_score REAL NOT NULL,
    decayed_score REAL NOT NULL,
    named_state TEXT NOT NULL,
    trial_count INTEGER NOT NULL,
    last_observed_at TEXT NOT NULL,
    half_life_ms INTEGER NOT NULL,
    snapshot_json TEXT NOT NULL,
    stamp_json TEXT NOT NULL
);

CREATE INDEX practice_mastery_snapshots_by_subject
ON practice_mastery_snapshots(workspace_id, subject_kind, subject_id, last_observed_at);

CREATE TABLE practice_bloom_snapshots (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    concept_id TEXT NOT NULL,
    observation_id TEXT NOT NULL,
    highest_demonstrated TEXT,
    snapshot_json TEXT NOT NULL,
    stamp_json TEXT NOT NULL,
    recorded_at TEXT NOT NULL
);

CREATE INDEX practice_bloom_snapshots_by_concept
ON practice_bloom_snapshots(workspace_id, concept_id, recorded_at);

CREATE TABLE practice_pattern_rules (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    pattern_id TEXT NOT NULL,
    required_clean INTEGER NOT NULL CHECK (required_clean BETWEEN 2 AND 5),
    updated_at TEXT NOT NULL,
    PRIMARY KEY (workspace_id, pattern_id)
);

CREATE TABLE practice_pattern_snapshots (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    pattern_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('watching', 'active', 'fading', 'resolved')),
    confirmed_count INTEGER NOT NULL,
    candidate_count INTEGER NOT NULL,
    clean_streak INTEGER NOT NULL,
    required_clean INTEGER NOT NULL,
    last_observed_at TEXT,
    snapshot_json TEXT NOT NULL,
    stamp_json TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (workspace_id, pattern_id)
);

CREATE TABLE practice_plans (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL CHECK (revision > 0),
    plan_json TEXT NOT NULL,
    stamp_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX practice_plans_by_workspace
ON practice_plans(workspace_id, updated_at DESC);

CREATE TABLE practice_attempt_states (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    plan_id TEXT NOT NULL REFERENCES practice_plans(id) ON DELETE CASCADE,
    slot_id TEXT NOT NULL,
    family_id TEXT NOT NULL,
    seed TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('open', 'solved', 'skipped')),
    instance_secret_json TEXT NOT NULL,
    attempt_state_json TEXT NOT NULL,
    stamp_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX practice_attempt_states_by_plan_slot
ON practice_attempt_states(plan_id, slot_id);

CREATE TABLE practice_action_receipts (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    idempotency_key TEXT NOT NULL,
    action_kind TEXT NOT NULL,
    request_digest TEXT NOT NULL,
    session_revision INTEGER NOT NULL,
    receipt_json TEXT NOT NULL,
    recorded_at TEXT NOT NULL,
    PRIMARY KEY (workspace_id, session_id, idempotency_key)
);

CREATE TABLE session_timing_checkpoints (
    session_id TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    elapsed_ms INTEGER NOT NULL DEFAULT 0 CHECK (elapsed_ms >= 0),
    attempt_elapsed_ms INTEGER NOT NULL DEFAULT 0 CHECK (attempt_elapsed_ms >= 0),
    target_ms INTEGER,
    checkpoint_sequence INTEGER NOT NULL DEFAULT 0 CHECK (checkpoint_sequence >= 0),
    active INTEGER NOT NULL DEFAULT 0 CHECK (active IN (0, 1)),
    measured_at TEXT NOT NULL,
    recovered INTEGER NOT NULL DEFAULT 0 CHECK (recovered IN (0, 1))
);

ALTER TABLE sessions ADD COLUMN plan_id TEXT;
ALTER TABLE sessions ADD COLUMN plan_revision INTEGER;
ALTER TABLE sessions ADD COLUMN session_revision INTEGER DEFAULT 1;
ALTER TABLE sessions ADD COLUMN summary_json TEXT;
ALTER TABLE sessions ADD COLUMN completed_at TEXT;
