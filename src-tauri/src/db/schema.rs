pub const LATEST_SCHEMA_VERSION: i64 = 4;

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_domain_schema",
        sql: include_str!("migrations/0001_initial.sql"),
    },
    Migration {
        version: 2,
        name: "practice_attempts",
        sql: include_str!("migrations/0002_practice.sql"),
    },
    Migration {
        version: 3,
        name: "session_practice_binding",
        sql: include_str!("migrations/0003_session_practice_binding.sql"),
    },
    Migration {
        version: 4,
        name: "session_activity",
        sql: include_str!("migrations/0004_session_activity.sql"),
    },
];
