CREATE TABLE practice_attempts (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    family_id TEXT NOT NULL,
    seed INTEGER NOT NULL,
    instance_json TEXT NOT NULL,
    hints_revealed INTEGER NOT NULL DEFAULT 0 CHECK (hints_revealed >= 0),
    status TEXT NOT NULL CHECK (status IN ('open', 'solved')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX practice_attempts_by_workspace
ON practice_attempts(workspace_id, status);

CREATE TABLE practice_submissions (
    id TEXT PRIMARY KEY,
    attempt_id TEXT NOT NULL REFERENCES practice_attempts(id) ON DELETE CASCADE,
    response_json TEXT NOT NULL,
    correct INTEGER NOT NULL CHECK (correct IN (0, 1)),
    submitted_at TEXT NOT NULL
);

CREATE INDEX practice_submissions_by_attempt
ON practice_submissions(attempt_id, submitted_at);
