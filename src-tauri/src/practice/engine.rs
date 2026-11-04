#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::collections::{BTreeMap, HashSet};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;
use uuid::Uuid;

use crate::capabilities::math_verify::{
    verify_v2, ContentRefInput, VerifyV2Request, VerifyV2Result, VerifyVerdict,
};
use crate::knowledge::{
    KnowledgePackage, ProblemFamily, ProblemFamilyStatus, ProblemInstance, ResolvedSolution,
};

use super::error::PracticeError;
use super::store::PracticeStore;
use super::types::{
    AttemptState, Attribution, Available, BloomEvidence, ConceptEvidence, ContentRef,
    CreatePlanInput, DecayDetail, DifficultyHistoryEntry, EditPatternRuleInput,
    EvidenceReadRequest, MasterySnapshot, PatternDetail, PatternDetailReadRequest,
    PatternListResponse, PatternResolutionInfo, PatternSessionBucket, PatternSnapshot,
    PatternsReadRequest, Plan, PlanChange, PlanGoalSnapshot, PlanPhase, PlanQueueSlot, PolicyRef,
    PracticeDifficulty, PrerequisiteReadiness, ProviderStampRef, Readiness, SessionActionInput,
    SessionActionReceipt, SessionHistoryEntry, SessionQuestionRow, SessionSummary, SessionTiming,
    Stamp, UnavailableReason, VerificationDto, VerificationObjectiveDto,
};

pub const DEFAULT_POLICY_ID: &str = "policy.reid_default";
pub const DEFAULT_POLICY_REVISION: u32 = 1;
pub const DEFAULT_POLICY_DIGEST: &str =
    "sha256:9b5781d0a8a3e5a638f18ef40219c6a09446e0470b14eb93eb5f07c7bf01b102";

const TRIAL_WEIGHT: f64 = 0.25;
const PARTIAL_CREDIT: f64 = 0.50;
const HALF_LIFE_MS: i64 = 1_209_600_000; // 14 days
const READINESS_THRESHOLD: f64 = 0.60;

pub fn default_policy_ref() -> PolicyRef {
    PolicyRef {
        id: DEFAULT_POLICY_ID.to_owned(),
        revision: DEFAULT_POLICY_REVISION,
        digest: DEFAULT_POLICY_DIGEST.to_owned(),
    }
}

pub fn package_content_ref(package: &KnowledgePackage) -> ContentRef {
    let pkg_id = package.id.as_str().to_owned();
    let pkg_ver = package.version.to_string();
    ContentRef {
        package_id: pkg_id.clone(),
        package_version: pkg_ver.clone(),
        schema_version: package.schema_version,
        digest: format!("sha256:{pkg_id}:{pkg_ver}:v{}", package.schema_version),
    }
}

pub fn make_stamp(
    workspace_id: &str,
    capability_id: &str,
    package: &KnowledgePackage,
    evidence_ids: Vec<String>,
    recorded_at: &str,
) -> Stamp {
    Stamp {
        schema_version: 1,
        workspace_id: workspace_id.to_owned(),
        provider: ProviderStampRef {
            module_id: "org.axiom.practice".to_owned(),
            module_version: "0.1.0".to_owned(),
            capability_id: capability_id.to_owned(),
            capability_version: 1,
        },
        policy_ref: Some(default_policy_ref()),
        content_ref: Some(package_content_ref(package)),
        evidence_ids,
        recorded_at: recorded_at.to_owned(),
    }
}

pub fn named_mastery_state(score: f64, trial_count: u32) -> &'static str {
    if score >= 0.85 && trial_count >= 3 {
        "Mastered"
    } else if score >= 0.70 {
        "Strong"
    } else if score >= 0.45 {
        "Familiar"
    } else {
        "Developing"
    }
}

pub fn apply_decay(raw_score: f64, elapsed_ms: i64) -> f64 {
    if elapsed_ms <= 0 {
        return raw_score.clamp(0.0, 1.0);
    }
    let exponent = -(elapsed_ms as f64) / (HALF_LIFE_MS as f64);
    (raw_score * 2.0_f64.powf(exponent)).clamp(0.0, 1.0)
}

fn parse_rfc3339_ms(ts: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|dt| dt.timestamp_millis())
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn resolve_knowledge_concept_id(
    conn: &Connection,
    workspace_id: &str,
    concept_id: &str,
    package: &KnowledgePackage,
) -> Option<String> {
    if package.concepts.iter().any(|c| c.id.as_str() == concept_id) {
        return Some(concept_id.to_owned());
    }

    let mapped: Option<Option<String>> = conn
        .query_row(
            "SELECT knowledge_concept_id FROM concepts WHERE id = ?1 AND workspace_id = ?2",
            params![concept_id, workspace_id],
            |row| row.get(0),
        )
        .optional()
        .ok()?;
    if let Some(Some(k_id)) = mapped {
        return Some(k_id);
    }

    let name: Option<String> = conn
        .query_row(
            "SELECT name FROM concepts WHERE id = ?1 AND workspace_id = ?2",
            params![concept_id, workspace_id],
            |row| row.get(0),
        )
        .optional()
        .ok()?;
    if let Some(name) = name {
        let lower = name.to_lowercase();
        if lower.contains("shell") {
            return Some("shell.method_vertical_axis".to_owned());
        }
    }
    None
}

fn resolve_workspace_concept_id(
    conn: &Connection,
    workspace_id: &str,
    knowledge_or_core_id: &str,
) -> String {
    if let Ok(Some(id)) = conn
        .query_row(
            "SELECT id FROM concepts WHERE workspace_id = ?1 AND (id = ?2 OR knowledge_concept_id = ?2) LIMIT 1",
            params![workspace_id, knowledge_or_core_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
    {
        return id;
    }
    knowledge_or_core_id.to_owned()
}

pub fn read_concept_evidence(
    store: &PracticeStore,
    package: &KnowledgePackage,
    request: EvidenceReadRequest,
) -> Result<Vec<ConceptEvidence>, PracticeError> {
    let conn = store.lock_connection()?;
    let as_of_str = request.as_of.unwrap_or_else(now_iso);
    let as_of_ms =
        parse_rfc3339_ms(&as_of_str).unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

    let target_concept_ids = if request.concept_ids.is_empty() {
        let mut stmt = conn
            .prepare("SELECT id FROM concepts WHERE workspace_id = ?1 ORDER BY id")
            .map_err(|e| PracticeError::Storage(e.to_string()))?;
        let rows = stmt
            .query_map([&request.workspace_id], |row| row.get::<_, String>(0))
            .map_err(|e| PracticeError::Storage(e.to_string()))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| PracticeError::Storage(e.to_string()))?;
        if rows.is_empty() {
            package
                .concepts
                .iter()
                .map(|c| c.id.as_str().to_owned())
                .collect()
        } else {
            rows
        }
    } else {
        request.concept_ids
    };

    let mut results = Vec::with_capacity(target_concept_ids.len());
    for concept_id in target_concept_ids {
        let mastery = load_mastery_for_concept(
            &conn,
            &request.workspace_id,
            &concept_id,
            &as_of_str,
            as_of_ms,
            package,
        )?;
        let bloom = load_bloom_for_concept(&conn, &request.workspace_id, &concept_id, package)?;
        let readiness = load_readiness_for_concept(
            &conn,
            &request.workspace_id,
            &concept_id,
            &as_of_str,
            as_of_ms,
            package,
        )?;
        results.push(ConceptEvidence {
            concept_id,
            mastery,
            bloom,
            readiness,
        });
    }

    Ok(results)
}

fn load_mastery_for_concept(
    conn: &Connection,
    workspace_id: &str,
    concept_id: &str,
    as_of_str: &str,
    as_of_ms: i64,
    package: &KnowledgePackage,
) -> Result<Available<MasterySnapshot>, PracticeError> {
    let k_concept_id = resolve_knowledge_concept_id(conn, workspace_id, concept_id, package)
        .unwrap_or_else(|| concept_id.to_owned());

    let row: Option<(String, String, i64)> = conn
        .query_row(
            "SELECT snapshot_json, stamp_json, trial_count
             FROM practice_mastery_snapshots
             WHERE workspace_id = ?1 AND subject_kind = 'concept' AND (subject_id = ?2 OR subject_id = ?3)
               AND last_observed_at <= ?4
             ORDER BY last_observed_at DESC, rowid DESC LIMIT 1",
            params![workspace_id, concept_id, k_concept_id, as_of_str],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let Some((snapshot_json, stamp_json, trial_count)) = row else {
        return Ok(Available::Unavailable {
            reason: UnavailableReason::NoEvidence,
            last_known: None,
        });
    };

    let mut snap: MasterySnapshot =
        serde_json::from_str(&snapshot_json).map_err(|e| PracticeError::Storage(e.to_string()))?;
    let stamp: Stamp =
        serde_json::from_str(&stamp_json).map_err(|e| PracticeError::Storage(e.to_string()))?;

    snap.concept_id = concept_id.to_owned();
    if let Some(obs_ms) = parse_rfc3339_ms(&snap.last_practiced_at) {
        let elapsed_ms = (as_of_ms - obs_ms).max(0);
        if elapsed_ms > 0 {
            let decayed = apply_decay(snap.score, elapsed_ms);
            let amount = (snap.score - decayed).max(0.0);
            if amount > 1e-6 {
                snap.decay = Some(DecayDetail {
                    from_snapshot_id: snap.id.clone(),
                    from_score: snap.score,
                    elapsed_ms,
                    amount,
                });
                snap.score = decayed;
                snap.named_state = named_mastery_state(decayed, trial_count as u32).to_owned();
                snap.as_of = as_of_str.to_owned();
            }
        }
    }

    Ok(Available::Available { value: snap, stamp })
}

fn load_bloom_for_concept(
    conn: &Connection,
    workspace_id: &str,
    concept_id: &str,
    package: &KnowledgePackage,
) -> Result<Available<BloomEvidence>, PracticeError> {
    let k_concept_id = resolve_knowledge_concept_id(conn, workspace_id, concept_id, package)
        .unwrap_or_else(|| concept_id.to_owned());

    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT snapshot_json, stamp_json
             FROM practice_bloom_snapshots
             WHERE workspace_id = ?1 AND (concept_id = ?2 OR concept_id = ?3)
             ORDER BY recorded_at DESC, rowid DESC LIMIT 1",
            params![workspace_id, concept_id, k_concept_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let Some((snapshot_json, stamp_json)) = row else {
        return Ok(Available::Unavailable {
            reason: UnavailableReason::NoEvidence,
            last_known: None,
        });
    };

    let bloom: BloomEvidence =
        serde_json::from_str(&snapshot_json).map_err(|e| PracticeError::Storage(e.to_string()))?;
    let stamp: Stamp =
        serde_json::from_str(&stamp_json).map_err(|e| PracticeError::Storage(e.to_string()))?;

    Ok(Available::Available {
        value: bloom,
        stamp,
    })
}

fn load_readiness_for_concept(
    conn: &Connection,
    workspace_id: &str,
    concept_id: &str,
    as_of_str: &str,
    as_of_ms: i64,
    package: &KnowledgePackage,
) -> Result<Available<Readiness>, PracticeError> {
    let mut stmt = conn
        .prepare(
            "SELECT target_concept_id FROM concept_edges
             WHERE source_concept_id = ?1 AND edge_kind = 'prerequisite'
             ORDER BY position",
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
    let mut prereq_ids = stmt
        .query_map([concept_id], |r| r.get::<_, String>(0))
        .map_err(|e| PracticeError::Storage(e.to_string()))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    if prereq_ids.is_empty() {
        if let Some(k_id) = resolve_knowledge_concept_id(conn, workspace_id, concept_id, package) {
            if let Some(k_concept) = package.concepts.iter().find(|c| c.id.as_str() == k_id) {
                prereq_ids = k_concept
                    .prerequisite_ids
                    .iter()
                    .map(|p| resolve_workspace_concept_id(conn, workspace_id, p.as_str()))
                    .collect();
            }
        }
    }

    let now = now_iso();
    if prereq_ids.is_empty() {
        let readiness = Readiness {
            state: "ready".to_owned(),
            prerequisites: Vec::new(),
            reason: "No prerequisite concepts required.".to_owned(),
        };
        return Ok(Available::Available {
            value: readiness,
            stamp: make_stamp(workspace_id, "practice.evidence", package, Vec::new(), &now),
        });
    }

    let mut prerequisites = Vec::with_capacity(prereq_ids.len());
    let mut evidence_ids = Vec::new();
    let mut any_not_ready = false;
    let mut any_unknown = false;

    for prereq_id in prereq_ids {
        match load_mastery_for_concept(
            conn,
            workspace_id,
            &prereq_id,
            as_of_str,
            as_of_ms,
            package,
        )? {
            Available::Available { value, .. } => {
                evidence_ids.push(value.id.clone());
                let st = if value.score >= READINESS_THRESHOLD {
                    "ready"
                } else {
                    any_not_ready = true;
                    "not-ready"
                };
                prerequisites.push(PrerequisiteReadiness {
                    concept_id: prereq_id,
                    snapshot_id: Some(value.id),
                    state: st.to_owned(),
                });
            }
            Available::Unavailable { .. } => {
                any_unknown = true;
                prerequisites.push(PrerequisiteReadiness {
                    concept_id: prereq_id,
                    snapshot_id: None,
                    state: "unknown".to_owned(),
                });
            }
        }
    }

    let (state, reason) = if any_not_ready {
        (
            "not-ready".to_owned(),
            "One or more prerequisite concepts are below the 0.60 readiness threshold.".to_owned(),
        )
    } else if any_unknown {
        (
            "unknown".to_owned(),
            "Prerequisite readiness requires observed practice evidence.".to_owned(),
        )
    } else {
        (
            "ready".to_owned(),
            "All prerequisite concepts meet the 0.60 readiness threshold.".to_owned(),
        )
    };

    Ok(Available::Available {
        value: Readiness {
            state,
            prerequisites,
            reason,
        },
        stamp: make_stamp(
            workspace_id,
            "practice.evidence",
            package,
            evidence_ids,
            &now,
        ),
    })
}

pub fn create_plan(
    store: &PracticeStore,
    package: &KnowledgePackage,
    input: CreatePlanInput,
) -> Result<Available<Plan>, PracticeError> {
    let conn = store.lock_connection()?;
    let now = now_iso();

    let difficulty = input.difficulty.unwrap_or(PracticeDifficulty::Steady);
    let target_ms = Some(
        input
            .target_ms
            .unwrap_or(1_500_000)
            .clamp(600_000, 2_700_000),
    );
    let requested_limit = input.question_limit.unwrap_or(5).clamp(3, 8) as usize;

    let goal_row: Option<(
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<i64>,
        Option<String>,
        String,
    )> = if let Some(ref gid) = input.goal_id {
        conn.query_row(
            "SELECT id, text, state, inferred_deadline, inferred_mastery_type, inferred_concept_scope, inferred_pacing, updated_at
             FROM goals WHERE id = ?1 AND workspace_id = ?2",
            params![gid, input.workspace_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?)),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?
    } else {
        conn.query_row(
            "SELECT id, text, state, inferred_deadline, inferred_mastery_type, inferred_concept_scope, inferred_pacing, updated_at
             FROM goals WHERE workspace_id = ?1 AND state = 'Guiding' LIMIT 1",
            params![input.workspace_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?)),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?
    };

    let (goal_id, goal_snapshot) =
        if let Some((id, text, state, deadline, mastery_type, concept_scope, pacing, updated_at)) =
            goal_row
        {
            (
                Some(id.clone()),
                Some(PlanGoalSnapshot {
                    id,
                    text,
                    state,
                    inferred: json!({
                        "deadline": deadline,
                        "masteryType": mastery_type,
                        "conceptScope": concept_scope,
                        "pacing": pacing,
                    }),
                    updated_at,
                }),
            )
        } else {
            (None, None)
        };

    let verified_families: Vec<&ProblemFamily> = package
        .problem_families
        .iter()
        .filter(|f| f.status == ProblemFamilyStatus::Verified)
        .collect();

    if verified_families.is_empty() {
        return Ok(Available::Unavailable {
            reason: UnavailableReason::ContentUnavailable,
            last_known: None,
        });
    }

    // Filter families if pattern_id or concept_ids specified
    let mut candidate_families = verified_families.clone();
    if let Some(ref pat_id) = input.pattern_id {
        let pat_filtered: Vec<&ProblemFamily> = candidate_families
            .iter()
            .copied()
            .filter(|f| {
                f.opportunities
                    .as_deref()
                    .unwrap_or(&[])
                    .iter()
                    .any(|o| o.pattern_id.as_str() == pat_id)
            })
            .collect();
        if !pat_filtered.is_empty() {
            candidate_families = pat_filtered;
        }
    }

    if !input.concept_ids.is_empty() {
        let resolved_k_ids: HashSet<String> = input
            .concept_ids
            .iter()
            .filter_map(|cid| {
                resolve_knowledge_concept_id(&conn, &input.workspace_id, cid, package)
            })
            .collect();
        if !resolved_k_ids.is_empty() {
            let filtered: Vec<&ProblemFamily> = candidate_families
                .iter()
                .copied()
                .filter(|f| resolved_k_ids.contains(f.concept_id.as_str()))
                .collect();
            if !filtered.is_empty() {
                candidate_families = filtered;
            }
        }
    }

    let warmup_count = ((requested_limit as f64) * 0.20).round().max(1.0) as usize;
    let combined_supported = candidate_families
        .iter()
        .any(|f| f.objective_ids.len() >= 2);
    let combined_count = if combined_supported && requested_limit >= 3 {
        ((requested_limit as f64) * 0.20).round().max(1.0) as usize
    } else {
        0
    };
    let stretch_count = requested_limit.saturating_sub(warmup_count + combined_count);

    let warmup_phase_id = "phase-warmup".to_owned();
    let stretch_phase_id = "phase-stretch".to_owned();
    let combined_phase_id = "phase-combined".to_owned();

    let mut phases = vec![
        PlanPhase {
            id: warmup_phase_id.clone(),
            name: "Warm-up".to_owned(),
            reason: "Recall core setup before full volume evaluation.".to_owned(),
            evidence_ids: Vec::new(),
        },
        PlanPhase {
            id: stretch_phase_id.clone(),
            name: "Stretch".to_owned(),
            reason: if let Some(ref pat_id) = input.pattern_id {
                format!("Targeted practice watching for pattern {pat_id}.")
            } else {
                "Build unassisted fluency on active syllabus concepts.".to_owned()
            },
            evidence_ids: Vec::new(),
        },
    ];
    if combined_count > 0 {
        phases.push(PlanPhase {
            id: combined_phase_id.clone(),
            name: "Combined".to_owned(),
            reason: "Multi-objective synthesis combining setup and evaluation.".to_owned(),
            evidence_ids: Vec::new(),
        });
    }

    let mut queue = Vec::with_capacity(requested_limit);
    let base_seeds: [u64; 8] = [17, 42, 73, 108, 256, 512, 1337, 90210];

    for idx in 0..requested_limit {
        let phase_id = if idx < warmup_count {
            &warmup_phase_id
        } else if idx < warmup_count + stretch_count {
            &stretch_phase_id
        } else {
            &combined_phase_id
        };

        let family = candidate_families[idx % candidate_families.len()];
        let ws_concept_id =
            resolve_workspace_concept_id(&conn, &input.workspace_id, family.concept_id.as_str());
        let seed = base_seeds[idx % base_seeds.len()] + (idx as u64) * 7;
        let slot_id = format!("slot-{}", idx + 1);
        let reason = match phase_id.as_str() {
            "phase-warmup" => {
                "Selected for Warm-up: verify shell radius and height setup.".to_owned()
            }
            "phase-combined" => {
                "Selected for Combined synthesis across setup and integration objectives."
                    .to_owned()
            }
            _ => {
                "Selected for Stretch: build unassisted accuracy on cylindrical shells.".to_owned()
            }
        };

        queue.push(PlanQueueSlot {
            id: slot_id,
            phase_id: phase_id.clone(),
            concept_id: ws_concept_id,
            objective_ids: family
                .objective_ids
                .iter()
                .map(|o| o.as_str().to_owned())
                .collect(),
            family_id: family.id.as_str().to_owned(),
            seed: seed.to_string(),
            attempt_id: None,
            state: "pending".to_owned(),
            reason,
            evidence_ids: Vec::new(),
            replaces_id: None,
        });
    }

    let mut limitations = Vec::new();
    let availability = if candidate_families.len() == 1 {
        limitations.push(
            "Single verified problem family available in package; seeds vary across queue slots."
                .to_owned(),
        );
        "limited".to_owned()
    } else {
        "complete".to_owned()
    };

    let safe_op: String = input
        .operation_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let plan_id = format!("plan-{safe_op}");
    let plan = Plan {
        id: plan_id.clone(),
        revision: 1,
        goal_id,
        goal_snapshot,
        target_ms,
        difficulty,
        policy_ref: default_policy_ref(),
        phases,
        queue,
        changes: Vec::new(),
        availability,
        limitations,
    };

    let stamp = make_stamp(
        &input.workspace_id,
        "practice.plan",
        package,
        Vec::new(),
        &now,
    );

    if !input.preview_only {
        let plan_json =
            serde_json::to_string(&plan).map_err(|e| PracticeError::Storage(e.to_string()))?;
        let stamp_json =
            serde_json::to_string(&stamp).map_err(|e| PracticeError::Storage(e.to_string()))?;

        conn.execute(
            "INSERT INTO practice_plans (id, workspace_id, revision, plan_json, stamp_json, created_at, updated_at)
             VALUES (?1, ?2, 1, ?3, ?4, ?5, ?5)
             ON CONFLICT(id) DO UPDATE SET plan_json = excluded.plan_json, updated_at = excluded.updated_at",
            params![plan_id, input.workspace_id, plan_json, stamp_json, now],
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
    }

    Ok(Available::Available { value: plan, stamp })
}

pub fn read_patterns(
    store: &PracticeStore,
    package: &KnowledgePackage,
    request: PatternsReadRequest,
) -> Result<PatternListResponse, PracticeError> {
    let conn = store.lock_connection()?;
    let now = now_iso();

    let total_answers: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM practice_observations WHERE workspace_id = ?1 AND outcome IN ('correct', 'incorrect', 'partial')",
            params![request.workspace_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let since_date: Option<String> = conn
        .query_row(
            "SELECT MIN(observed_at) FROM practice_observations WHERE workspace_id = ?1",
            params![request.workspace_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?
        .flatten();

    let mut stmt = conn
        .prepare(
            "SELECT snapshot_json FROM practice_pattern_snapshots
             WHERE workspace_id = ?1
             ORDER BY updated_at DESC, pattern_id ASC",
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let mut snapshots: Vec<PatternSnapshot> = stmt
        .query_map([&request.workspace_id], |r| {
            let s: String = r.get(0)?;
            Ok(s)
        })
        .map_err(|e| PracticeError::Storage(e.to_string()))?
        .filter_map(|r| r.ok())
        .filter_map(|s| serde_json::from_str::<PatternSnapshot>(&s).ok())
        .collect();

    // Ensure package patterns that aren't yet in DB are surfaced if requested or if package has v2 patterns
    for pat in &package.patterns {
        if !snapshots.iter().any(|s| s.pattern_id == pat.id.as_str()) {
            let req_clean =
                load_pattern_required_clean(&conn, &request.workspace_id, pat.id.as_str());
            snapshots.push(PatternSnapshot {
                id: format!("patsnap-{}-{}", request.workspace_id, pat.id.as_str()),
                pattern_id: pat.id.as_str().to_owned(),
                name: pat.name.clone(),
                concept_ids: Some(
                    pat.concept_ids
                        .iter()
                        .map(|c| c.as_str().to_owned())
                        .collect(),
                ),
                status: "Unknown".to_owned(),
                opportunities: 0,
                occurrences: 0,
                unknown_count: 0,
                clean_streak: 0,
                first_seen_at: None,
                last_seen_at: None,
                recent_attribution_ids: Vec::new(),
                rule_revision: 1,
                resolution: PatternResolutionInfo {
                    required_clean: req_clean,
                    eligible_clean: 0,
                    explanation: format!("0 / {req_clean} clean opportunities observed."),
                },
            });
        }
    }

    if let Some(ref status_filter) = request.status {
        if !status_filter.is_empty() && !status_filter.eq_ignore_ascii_case("all") {
            snapshots.retain(|s| s.status.eq_ignore_ascii_case(status_filter));
        }
    }

    if let Some(ref search) = request.search {
        let q = search.trim().to_lowercase();
        if !q.is_empty() {
            snapshots.retain(|s| {
                s.name.to_lowercase().contains(&q) || s.pattern_id.to_lowercase().contains(&q)
            });
        }
    }

    let stamp = make_stamp(
        &request.workspace_id,
        "practice.patterns",
        package,
        Vec::new(),
        &now,
    );

    let patterns = if snapshots.is_empty() && total_answers == 0 {
        Available::Unavailable {
            reason: UnavailableReason::NoEvidence,
            last_known: None,
        }
    } else {
        Available::Available {
            value: snapshots,
            stamp,
        }
    };

    Ok(PatternListResponse {
        patterns,
        total_answers: total_answers as u32,
        since_date,
        required_clean_default: 3,
        next_cursor: None,
    })
}

fn load_pattern_required_clean(conn: &Connection, workspace_id: &str, pattern_id: &str) -> u32 {
    conn.query_row(
        "SELECT required_clean FROM practice_pattern_rules WHERE workspace_id = ?1 AND pattern_id = ?2",
        params![workspace_id, pattern_id],
        |r| r.get::<_, i64>(0),
    )
    .optional()
    .ok()
    .flatten()
    .map(|v| v as u32)
    .unwrap_or(3)
}

pub fn read_pattern_detail(
    store: &PracticeStore,
    package: &KnowledgePackage,
    request: PatternDetailReadRequest,
) -> Result<Available<PatternDetail>, PracticeError> {
    let conn = store.lock_connection()?;
    let now = now_iso();

    let snap_row: Option<(String, String)> = conn
        .query_row(
            "SELECT snapshot_json, stamp_json FROM practice_pattern_snapshots
             WHERE workspace_id = ?1 AND pattern_id = ?2",
            params![request.workspace_id, request.pattern_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let (snapshot, stamp) = if let Some((s_json, st_json)) = snap_row {
        let s: PatternSnapshot =
            serde_json::from_str(&s_json).map_err(|e| PracticeError::Storage(e.to_string()))?;
        let st: Stamp =
            serde_json::from_str(&st_json).map_err(|e| PracticeError::Storage(e.to_string()))?;
        (s, st)
    } else if let Some(pat) = package
        .patterns
        .iter()
        .find(|p| p.id.as_str() == request.pattern_id)
    {
        let req_clean =
            load_pattern_required_clean(&conn, &request.workspace_id, &request.pattern_id);
        let s = PatternSnapshot {
            id: format!("patsnap-{}-{}", request.workspace_id, pat.id.as_str()),
            pattern_id: pat.id.as_str().to_owned(),
            name: pat.name.clone(),
            concept_ids: Some(
                pat.concept_ids
                    .iter()
                    .map(|c| c.as_str().to_owned())
                    .collect(),
            ),
            status: "Unknown".to_owned(),
            opportunities: 0,
            occurrences: 0,
            unknown_count: 0,
            clean_streak: 0,
            first_seen_at: None,
            last_seen_at: None,
            recent_attribution_ids: Vec::new(),
            rule_revision: 1,
            resolution: PatternResolutionInfo {
                required_clean: req_clean,
                eligible_clean: 0,
                explanation: format!("0 / {req_clean} clean opportunities observed."),
            },
        };
        let st = make_stamp(
            &request.workspace_id,
            "practice.patterns",
            package,
            Vec::new(),
            &now,
        );
        (s, st)
    } else {
        return Ok(Available::Unavailable {
            reason: UnavailableReason::NoEvidence,
            last_known: None,
        });
    };

    let mut stmt = conn
        .prepare(
            "SELECT attribution_json FROM practice_attributions
             WHERE workspace_id = ?1 AND pattern_id = ?2
             ORDER BY recorded_at DESC",
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
    let attributions: Vec<Attribution> = stmt
        .query_map(params![request.workspace_id, request.pattern_id], |r| {
            r.get::<_, String>(0)
        })
        .map_err(|e| PracticeError::Storage(e.to_string()))?
        .filter_map(|r| r.ok())
        .filter_map(|s| serde_json::from_str::<Attribution>(&s).ok())
        .collect();

    let mut bucket_map: BTreeMap<String, PatternSessionBucket> = BTreeMap::new();
    for attr in &attributions {
        let sid = attr
            .session_id
            .clone()
            .unwrap_or_else(|| "session-recorded".to_owned());
        let entry = bucket_map
            .entry(sid.clone())
            .or_insert_with(|| PatternSessionBucket {
                session_id: sid,
                recorded_at: attr.recorded_at.clone().unwrap_or_else(|| now.clone()),
                opportunities: 0,
                occurrences: 0,
                clean_count: 0,
                unknown_count: 0,
            });
        entry.opportunities += 1;
        match attr.outcome.as_str() {
            "observed" => entry.occurrences += 1,
            "clean" => entry.clean_count += 1,
            _ => entry.unknown_count += 1,
        }
    }

    let explanation = package
        .patterns
        .iter()
        .find(|p| p.id.as_str() == request.pattern_id)
        .map(|p| p.explanation.clone())
        .unwrap_or_else(|| {
            format!(
                "Pattern {} tracked across {} eligible opportunities.",
                snapshot.name, snapshot.opportunities
            )
        });

    let remaining = snapshot
        .resolution
        .required_clean
        .saturating_sub(snapshot.clean_streak);
    let next_action = if remaining == 0 {
        "Pattern is resolved under the current rule.".to_owned()
    } else {
        format!(
            "Complete {remaining} more consecutive clean unassisted problem{} on this pattern to mark it Resolved.",
            if remaining == 1 { "" } else { "s" }
        )
    };

    Ok(Available::Available {
        value: PatternDetail {
            snapshot,
            explanation,
            next_action,
            attributions,
            session_buckets: bucket_map.into_values().collect(),
        },
        stamp,
    })
}

pub fn edit_pattern_rule(
    store: &PracticeStore,
    package: &KnowledgePackage,
    input: EditPatternRuleInput,
) -> Result<Available<PatternSnapshot>, PracticeError> {
    if !(2..=5).contains(&input.required_clean) {
        return Err(PracticeError::Storage(format!(
            "invalid-input: required_clean {} must be between 2 and 5",
            input.required_clean
        )));
    }

    let conn = store.lock_connection()?;
    let now = now_iso();

    if !input.preview_only {
        conn.execute(
            "INSERT INTO practice_pattern_rules (workspace_id, pattern_id, required_clean, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(workspace_id, pattern_id) DO UPDATE SET required_clean = excluded.required_clean, updated_at = excluded.updated_at",
            params![input.workspace_id, input.pattern_id, input.required_clean as i64, now],
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
    }

    let existing_row: Option<(String, String)> = conn
        .query_row(
            "SELECT snapshot_json, stamp_json FROM practice_pattern_snapshots
             WHERE workspace_id = ?1 AND pattern_id = ?2",
            params![input.workspace_id, input.pattern_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let mut snap = if let Some((s_json, _)) = existing_row {
        serde_json::from_str::<PatternSnapshot>(&s_json)
            .map_err(|e| PracticeError::Storage(e.to_string()))?
    } else {
        let name = package
            .patterns
            .iter()
            .find(|p| p.id.as_str() == input.pattern_id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| input.pattern_id.clone());
        PatternSnapshot {
            id: format!("patsnap-{}-{}", input.workspace_id, input.pattern_id),
            pattern_id: input.pattern_id.clone(),
            name,
            concept_ids: None,
            status: "Unknown".to_owned(),
            opportunities: 0,
            occurrences: 0,
            unknown_count: 0,
            clean_streak: 0,
            first_seen_at: None,
            last_seen_at: None,
            recent_attribution_ids: Vec::new(),
            rule_revision: input.rule_revision,
            resolution: PatternResolutionInfo {
                required_clean: input.required_clean,
                eligible_clean: 0,
                explanation: String::new(),
            },
        }
    };

    // P4: Rule edit starts a new clean streak from the new rule revision
    snap.rule_revision += 1;
    snap.clean_streak = 0;
    snap.resolution.required_clean = input.required_clean;
    snap.resolution.eligible_clean = 0;
    snap.resolution.explanation = format!(
        "Rule updated to require {} consecutive clean unassisted attempts (streak reset at revision {}).",
        input.required_clean, snap.rule_revision
    );
    if snap.occurrences >= 2 {
        snap.status = "Recurring".to_owned();
    } else if snap.occurrences == 1 {
        snap.status = "Improving".to_owned();
    }

    let stamp = make_stamp(
        &input.workspace_id,
        "practice.patterns",
        package,
        Vec::new(),
        &now,
    );

    if !input.preview_only {
        let snap_json =
            serde_json::to_string(&snap).map_err(|e| PracticeError::Storage(e.to_string()))?;
        let stamp_json =
            serde_json::to_string(&stamp).map_err(|e| PracticeError::Storage(e.to_string()))?;
        let db_status = match snap.status.as_str() {
            "Recurring" => "active",
            "Improving" => "fading",
            "Resolved" => "resolved",
            _ => "watching",
        };
        conn.execute(
            "INSERT INTO practice_pattern_snapshots (
                id, workspace_id, pattern_id, status, confirmed_count, candidate_count,
                clean_streak, required_clean, last_observed_at, snapshot_json, stamp_json, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(workspace_id, pattern_id) DO UPDATE SET
                status = excluded.status,
                clean_streak = excluded.clean_streak,
                required_clean = excluded.required_clean,
                snapshot_json = excluded.snapshot_json,
                stamp_json = excluded.stamp_json,
                updated_at = excluded.updated_at",
            params![
                snap.id,
                input.workspace_id,
                input.pattern_id,
                db_status,
                snap.occurrences as i64,
                snap.clean_streak as i64,
                input.required_clean as i64,
                snap.last_seen_at,
                snap_json,
                stamp_json,
                now,
            ],
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
    }

    Ok(Available::Available { value: snap, stamp })
}

pub fn execute_session_action(
    store: &PracticeStore,
    package: &KnowledgePackage,
    input: SessionActionInput,
    precomputed_verification: Option<VerifyV2Result>,
) -> Result<SessionActionReceipt, PracticeError> {
    let mut conn = store.lock_connection()?;
    let now = now_iso();
    let req_digest = format!("{}:{}", input.action, input.payload);

    // 1. Check idempotency receipt outside/before revision check ( skipped for 'read' )
    if input.action != "read" {
        let existing_receipt: Option<(String, String)> = conn
            .query_row(
                "SELECT request_digest, receipt_json FROM practice_action_receipts
                 WHERE workspace_id = ?1 AND session_id = ?2 AND idempotency_key = ?3",
                params![input.workspace_id, input.session_id, input.operation_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(|e| PracticeError::Storage(e.to_string()))?;

        if let Some((saved_digest, receipt_json)) = existing_receipt {
            if saved_digest != req_digest {
                return Err(PracticeError::Storage(
                    "operation-conflict: idempotency key reused with different payload".to_owned(),
                ));
            }
            return serde_json::from_str::<SessionActionReceipt>(&receipt_json)
                .map_err(|e| PracticeError::Storage(e.to_string()));
        }
    }

    let tx = conn
        .transaction()
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let receipt = match input.action.as_str() {
        "read" => handle_read_action(&tx, package, &input, &now)?,
        "start" => handle_start_action(&tx, package, &input, &now)?,
        "resume" => handle_resume_or_pause_action(&tx, package, &input, true, &now)?,
        "pause" => handle_resume_or_pause_action(&tx, package, &input, false, &now)?,
        "save-working" => handle_save_working_action(&tx, package, &input, &now)?,
        "hint" => handle_hint_action(&tx, package, &input, &now)?,
        "reveal-solution" => handle_reveal_solution_action(&tx, package, &input, &now)?,
        "check" => handle_check_action(&tx, package, &input, precomputed_verification, &now)?,
        "skip" => handle_skip_or_advance_action(&tx, package, &input, true, &now)?,
        "advance" => handle_skip_or_advance_action(&tx, package, &input, false, &now)?,
        "edit-plan" => handle_edit_plan_action(&tx, package, &input, &now)?,
        "end" => handle_end_action(&tx, package, &input, &now)?,
        other => {
            return Err(PracticeError::Storage(format!(
                "invalid-input: unsupported session action {other:?}"
            )))
        }
    };

    if input.action != "read" {
        let receipt_json =
            serde_json::to_string(&receipt).map_err(|e| PracticeError::Storage(e.to_string()))?;
        tx.execute(
            "INSERT INTO practice_action_receipts (
                workspace_id, session_id, idempotency_key, action_kind,
                request_digest, session_revision, receipt_json, recorded_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                input.workspace_id,
                input.session_id,
                input.operation_id,
                input.action,
                req_digest,
                receipt.revision as i64,
                receipt_json,
                now,
            ],
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
    }

    tx.commit()
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    Ok(receipt)
}

fn load_or_init_timing(
    conn: &Connection,
    workspace_id: &str,
    session_id: &str,
    target_ms: Option<i64>,
    elapsed_delta_ms: Option<i64>,
    attempt_delta_ms: Option<i64>,
    active: Option<bool>,
    reset_attempt_elapsed: bool,
    now: &str,
) -> Result<Available<SessionTiming>, PracticeError> {
    let existing: Option<(i64, i64, Option<i64>, i64, i64, String, i64)> = conn
        .query_row(
            "SELECT elapsed_ms, attempt_elapsed_ms, target_ms, checkpoint_sequence, active, measured_at, recovered
             FROM session_timing_checkpoints WHERE session_id = ?1 AND workspace_id = ?2",
            params![session_id, workspace_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let d_elapsed = elapsed_delta_ms.unwrap_or(0).clamp(0, 60_000);
    let d_attempt = attempt_delta_ms.unwrap_or(d_elapsed).clamp(0, 60_000);

    let timing = if let Some((el, att_el, tgt, seq, act, _, rec)) = existing {
        let new_el = el + d_elapsed;
        let new_att = if reset_attempt_elapsed {
            0
        } else {
            att_el + d_attempt
        };
        let new_seq = if d_elapsed > 0 || active.is_some() || reset_attempt_elapsed {
            seq + 1
        } else {
            seq
        };
        let new_act = active.unwrap_or(act != 0);
        let new_tgt = target_ms.or(tgt);
        conn.execute(
            "UPDATE session_timing_checkpoints
             SET elapsed_ms = ?2, attempt_elapsed_ms = ?3, target_ms = ?4,
                 checkpoint_sequence = ?5, active = ?6, measured_at = ?7
             WHERE session_id = ?1",
            params![
                session_id,
                new_el,
                new_att,
                new_tgt,
                new_seq,
                i64::from(new_act),
                now,
            ],
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
        SessionTiming {
            elapsed_ms: new_el,
            attempt_elapsed_ms: new_att,
            target_ms: new_tgt,
            checkpoint_sequence: new_seq as u32,
            active: new_act,
            measured_at: now.to_owned(),
            recovered: rec != 0,
        }
    } else {
        let new_act = active.unwrap_or(true);
        conn.execute(
            "INSERT INTO session_timing_checkpoints (
                session_id, workspace_id, elapsed_ms, attempt_elapsed_ms, target_ms,
                checkpoint_sequence, active, measured_at, recovered
             ) VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, 0)",
            params![
                session_id,
                workspace_id,
                d_elapsed,
                if reset_attempt_elapsed { 0 } else { d_attempt },
                target_ms,
                i64::from(new_act),
                now,
            ],
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
        SessionTiming {
            elapsed_ms: d_elapsed,
            attempt_elapsed_ms: if reset_attempt_elapsed { 0 } else { d_attempt },
            target_ms,
            checkpoint_sequence: 1,
            active: new_act,
            measured_at: now.to_owned(),
            recovered: false,
        }
    };

    let stamp = Stamp {
        schema_version: 1,
        workspace_id: workspace_id.to_owned(),
        provider: ProviderStampRef {
            module_id: "org.axiom.practice".to_owned(),
            module_version: "0.1.0".to_owned(),
            capability_id: "practice.session".to_owned(),
            capability_version: 1,
        },
        policy_ref: Some(default_policy_ref()),
        content_ref: None,
        evidence_ids: Vec::new(),
        recorded_at: now.to_owned(),
    };

    Ok(Available::Available {
        value: timing,
        stamp,
    })
}

fn ensure_attempt_for_slot(
    conn: &Connection,
    workspace_id: &str,
    session_id: &str,
    plan_id: &str,
    difficulty: PracticeDifficulty,
    slot: &mut PlanQueueSlot,
    package: &KnowledgePackage,
    now: &str,
) -> Result<AttemptState, PracticeError> {
    if let Some(ref att_id) = slot.attempt_id {
        if let Some(state) = load_attempt_state(conn, workspace_id, att_id)? {
            return Ok(state);
        }
    }

    let family = package
        .problem_families
        .iter()
        .find(|f| f.id.as_str() == slot.family_id)
        .unwrap_or_else(|| &package.problem_families[0]);

    let seed_u64 = slot.seed.parse::<u64>().unwrap_or(42);
    let instance = crate::generation::generate_problem_instance(family, seed_u64)?;
    let attempt_id = format!("attempt-{}", Uuid::new_v4());

    let bloom_target = family
        .objective_ids
        .iter()
        .filter_map(|oid| package.objectives.iter().find(|o| o.id == *oid))
        .filter_map(|o| o.bloom_level)
        .map(|b| b.as_str().to_owned())
        .next()
        .or_else(|| Some("apply".to_owned()));

    let state = AttemptState {
        id: attempt_id.clone(),
        slot_id: slot.id.clone(),
        status: "open".to_owned(),
        instance_ref: format!("{}:{}", family.id.as_str(), seed_u64),
        prompt: instance.prompt.clone(),
        response_type: family.response_type,
        restrictions: Vec::new(),
        bloom_target,
        draft_revision: 0,
        answer_text: String::new(),
        working_text: String::new(),
        hints_total: instance.hints.len() as u32,
        hints_revealed: 0,
        hint_budget: difficulty.hint_budget().min(instance.hints.len() as u32),
        revealed_hints: Some(Vec::new()),
        canonical_solution_display: None,
        worked_solution: None,
        submission_ids: Vec::new(),
        last_feedback_id: None,
        last_verification: None,
    };

    let instance_json =
        serde_json::to_string(&instance).map_err(|e| PracticeError::Storage(e.to_string()))?;
    let state_json =
        serde_json::to_string(&state).map_err(|e| PracticeError::Storage(e.to_string()))?;
    let stamp = make_stamp(workspace_id, "practice.session", package, Vec::new(), now);
    let stamp_json =
        serde_json::to_string(&stamp).map_err(|e| PracticeError::Storage(e.to_string()))?;

    // Also insert into practice_attempts so legacy @1 describeAttempt/requestHint/evaluateAttempt work seamlessly!
    conn.execute(
        "INSERT INTO practice_attempts (
            id, workspace_id, family_id, seed, instance_json, hints_revealed, status, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, 0, 'open', ?6, ?6)",
        params![
            attempt_id,
            workspace_id,
            family.id.as_str(),
            seed_u64 as i64,
            instance_json,
            now
        ],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    conn.execute(
        "INSERT INTO practice_attempt_states (
            id, workspace_id, session_id, plan_id, slot_id, family_id, seed, status,
            instance_secret_json, attempt_state_json, stamp_json, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'open', ?8, ?9, ?10, ?11, ?11)",
        params![
            attempt_id,
            workspace_id,
            session_id,
            plan_id,
            slot.id,
            family.id.as_str(),
            slot.seed,
            instance_json,
            state_json,
            stamp_json,
            now,
        ],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    slot.attempt_id = Some(attempt_id);
    if slot.state == "pending" {
        slot.state = "current".to_owned();
    }

    Ok(state)
}

pub fn load_attempt_state(
    conn: &Connection,
    workspace_id: &str,
    attempt_id: &str,
) -> Result<Option<AttemptState>, PracticeError> {
    let json_opt: Option<String> = conn
        .query_row(
            "SELECT attempt_state_json FROM practice_attempt_states WHERE id = ?1 AND workspace_id = ?2",
            params![attempt_id, workspace_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    json_opt
        .map(|s| {
            serde_json::from_str::<AttemptState>(&s)
                .map_err(|e| PracticeError::Storage(e.to_string()))
        })
        .transpose()
}

pub fn load_attempt_secret_instance(
    conn: &Connection,
    workspace_id: &str,
    attempt_id: &str,
) -> Result<Option<(ProblemInstance, String)>, PracticeError> {
    let row: Option<(String, String)> = conn
        .query_row(
            "SELECT instance_secret_json, family_id FROM practice_attempt_states WHERE id = ?1 AND workspace_id = ?2",
            params![attempt_id, workspace_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    if let Some((inst_json, fam_id)) = row {
        let inst: ProblemInstance =
            serde_json::from_str(&inst_json).map_err(|e| PracticeError::Storage(e.to_string()))?;
        return Ok(Some((inst, fam_id)));
    }
    Ok(None)
}

fn save_attempt_state(
    conn: &Connection,
    workspace_id: &str,
    state: &AttemptState,
    now: &str,
) -> Result<(), PracticeError> {
    let state_json =
        serde_json::to_string(state).map_err(|e| PracticeError::Storage(e.to_string()))?;
    conn.execute(
        "UPDATE practice_attempt_states
         SET status = ?3, attempt_state_json = ?4, updated_at = ?5
         WHERE id = ?1 AND workspace_id = ?2",
        params![state.id, workspace_id, state.status, state_json, now],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let legacy_status = if state.status == "solved" {
        "solved"
    } else {
        "open"
    };
    let _ = conn.execute(
        "UPDATE practice_attempts SET status = ?2, hints_revealed = ?3, updated_at = ?4 WHERE id = ?1",
        params![state.id, legacy_status, state.hints_revealed as i64, now],
    );
    Ok(())
}

fn load_plan_by_id(
    conn: &Connection,
    workspace_id: &str,
    plan_id: &str,
) -> Result<Plan, PracticeError> {
    let plan_json: String = conn
        .query_row(
            "SELECT plan_json FROM practice_plans WHERE id = ?1 AND workspace_id = ?2",
            params![plan_id, workspace_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?
        .ok_or_else(|| PracticeError::Storage(format!("not-found: plan {plan_id} not found")))?;

    serde_json::from_str::<Plan>(&plan_json).map_err(|e| PracticeError::Storage(e.to_string()))
}

fn save_plan(
    conn: &Connection,
    workspace_id: &str,
    plan: &Plan,
    package: &KnowledgePackage,
    now: &str,
) -> Result<(), PracticeError> {
    let plan_json =
        serde_json::to_string(plan).map_err(|e| PracticeError::Storage(e.to_string()))?;
    let stamp = make_stamp(workspace_id, "practice.plan", package, Vec::new(), now);
    let stamp_json =
        serde_json::to_string(&stamp).map_err(|e| PracticeError::Storage(e.to_string()))?;
    conn.execute(
        "INSERT INTO practice_plans (id, workspace_id, revision, plan_json, stamp_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
         ON CONFLICT(id) DO UPDATE SET
            revision = excluded.revision,
            plan_json = excluded.plan_json,
            stamp_json = excluded.stamp_json,
            updated_at = excluded.updated_at",
        params![plan.id, workspace_id, plan.revision as i64, plan_json, stamp_json, now],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;
    Ok(())
}

fn load_or_create_session_plan(
    conn: &Connection,
    workspace_id: &str,
    session_id: &str,
    package: &KnowledgePackage,
    now: &str,
) -> Result<(Plan, u32, Option<String>, Option<String>), PracticeError> {
    let row: Option<(Option<String>, Option<i64>, Option<String>, Option<String>, String)> = conn
        .query_row(
            "SELECT plan_id, COALESCE(session_revision, 1), current_attempt_id, summary_json, concept_id
             FROM sessions WHERE id = ?1 AND workspace_id = ?2",
            params![session_id, workspace_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let (plan_id_opt, rev_opt, curr_att, summary_json, concept_id) = row.ok_or_else(|| {
        PracticeError::Storage(format!("not-found: session {session_id} not found"))
    })?;
    let rev = rev_opt.unwrap_or(1) as u32;

    if let Some(plan_id) = plan_id_opt {
        if let Ok(plan) = load_plan_by_id(conn, workspace_id, &plan_id) {
            return Ok((plan, rev, curr_att, summary_json));
        }
    }

    // Create a default plan for this session if it was started via legacy startSession
    let family = &package.problem_families[0];
    let base_seeds: [u64; 5] = [17, 42, 73, 108, 256];
    let mut queue = Vec::with_capacity(5);
    for (idx, seed) in base_seeds.iter().enumerate() {
        let phase_id = match idx {
            0 => "phase-warmup",
            4 => "phase-combined",
            _ => "phase-stretch",
        };
        queue.push(PlanQueueSlot {
            id: format!("slot-{}", idx + 1),
            phase_id: phase_id.to_owned(),
            concept_id: concept_id.clone(),
            objective_ids: family
                .objective_ids
                .iter()
                .map(|o| o.as_str().to_owned())
                .collect(),
            family_id: family.id.as_str().to_owned(),
            seed: seed.to_string(),
            attempt_id: if idx == 0 { curr_att.clone() } else { None },
            state: if idx == 0 {
                "current".to_owned()
            } else {
                "pending".to_owned()
            },
            reason: match idx {
                0 => "Selected for Warm-up: verify shell radius and height setup.".to_owned(),
                4 => "Selected for Combined synthesis across setup and integration objectives."
                    .to_owned(),
                _ => "Selected for Stretch: build unassisted accuracy on cylindrical shells."
                    .to_owned(),
            },
            evidence_ids: Vec::new(),
            replaces_id: None,
        });
    }

    let plan_id = format!("plan-session-{session_id}");
    let plan = Plan {
        id: plan_id.clone(),
        revision: 1,
        goal_id: None,
        goal_snapshot: None,
        target_ms: Some(1_500_000),
        difficulty: PracticeDifficulty::Steady,
        policy_ref: default_policy_ref(),
        phases: vec![
            PlanPhase {
                id: "phase-warmup".to_owned(),
                name: "Warm-up".to_owned(),
                reason: "Recall core setup before full volume evaluation.".to_owned(),
                evidence_ids: Vec::new(),
            },
            PlanPhase {
                id: "phase-stretch".to_owned(),
                name: "Stretch".to_owned(),
                reason: "Build unassisted fluency on active syllabus concepts.".to_owned(),
                evidence_ids: Vec::new(),
            },
            PlanPhase {
                id: "phase-combined".to_owned(),
                name: "Combined".to_owned(),
                reason: "Multi-objective synthesis combining setup and evaluation.".to_owned(),
                evidence_ids: Vec::new(),
            },
        ],
        queue,
        changes: Vec::new(),
        availability: "limited".to_owned(),
        limitations: vec![
            "Single verified problem family available in package; seeds vary across queue slots."
                .to_owned(),
        ],
    };

    save_plan(conn, workspace_id, &plan, package, now)?;
    conn.execute(
        "UPDATE sessions SET plan_id = ?2, plan_revision = 1 WHERE id = ?1",
        params![session_id, plan_id],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    Ok((plan, rev, curr_att, summary_json))
}

fn unavailable_summary() -> Available<SessionSummary> {
    Available::Unavailable {
        reason: UnavailableReason::NoEvidence,
        last_known: None,
    }
}

fn parse_saved_summary(
    workspace_id: &str,
    package: &KnowledgePackage,
    summary_json: Option<String>,
    now: &str,
) -> Available<SessionSummary> {
    if let Some(s_json) = summary_json {
        if let Ok(summary) = serde_json::from_str::<SessionSummary>(&s_json) {
            return Available::Available {
                value: summary,
                stamp: make_stamp(workspace_id, "practice.session", package, Vec::new(), now),
            };
        }
    }
    unavailable_summary()
}

fn handle_read_action(
    conn: &Connection,
    package: &KnowledgePackage,
    input: &SessionActionInput,
    now: &str,
) -> Result<SessionActionReceipt, PracticeError> {
    let (mut plan, rev, _curr_att, summary_json) =
        load_or_create_session_plan(conn, &input.workspace_id, &input.session_id, package, now)?;

    let active_idx = plan
        .queue
        .iter()
        .position(|s| s.state == "current")
        .or_else(|| plan.queue.iter().position(|s| s.state == "pending"));

    let attempt = if let Some(idx) = active_idx {
        let difficulty = plan.difficulty;
        let plan_id = plan.id.clone();
        let state = ensure_attempt_for_slot(
            conn,
            &input.workspace_id,
            &input.session_id,
            &plan_id,
            difficulty,
            &mut plan.queue[idx],
            package,
            now,
        )?;
        save_plan(conn, &input.workspace_id, &plan, package, now)?;
        conn.execute(
            "UPDATE sessions SET current_attempt_id = ?2, problem_index = ?3, problem_count = ?4 WHERE id = ?1",
            params![
                input.session_id,
                state.id,
                (idx + 1) as i64,
                plan.queue.len() as i64
            ],
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
        Some(state)
    } else {
        None
    };

    let timing = load_or_init_timing(
        conn,
        &input.workspace_id,
        &input.session_id,
        plan.target_ms,
        input.elapsed_delta_ms,
        input.attempt_elapsed_delta_ms,
        None,
        false,
        now,
    )?;

    let summary = parse_saved_summary(&input.workspace_id, package, summary_json, now);

    Ok(SessionActionReceipt {
        operation_id: input.operation_id.clone(),
        session_id: input.session_id.clone(),
        revision: rev,
        event_ids: Vec::new(),
        attempt,
        mastery_changes: Vec::new(),
        bloom_changes: Vec::new(),
        pattern_changes: Vec::new(),
        plan,
        timing,
        summary,
    })
}

fn handle_start_action(
    conn: &Connection,
    package: &KnowledgePackage,
    input: &SessionActionInput,
    now: &str,
) -> Result<SessionActionReceipt, PracticeError> {
    let plan_id = input
        .payload
        .get("planId")
        .or_else(|| input.payload.get("plan_id"))
        .and_then(|v| v.as_str())
        .map(str::to_owned);

    let mut plan = if let Some(ref pid) = plan_id {
        load_plan_by_id(conn, &input.workspace_id, pid)?
    } else {
        let (p, _, _, _) = load_or_create_session_plan(
            conn,
            &input.workspace_id,
            &input.session_id,
            package,
            now,
        )?;
        p
    };

    if plan.queue.is_empty() {
        return Err(PracticeError::Storage(
            "content-unavailable: plan queue is empty".to_owned(),
        ));
    }

    let first_idx = plan
        .queue
        .iter()
        .position(|s| s.state == "current")
        .or_else(|| plan.queue.iter().position(|s| s.state == "pending"))
        .unwrap_or(0);

    let difficulty = plan.difficulty;
    let pid = plan.id.clone();
    let attempt = ensure_attempt_for_slot(
        conn,
        &input.workspace_id,
        &input.session_id,
        &pid,
        difficulty,
        &mut plan.queue[first_idx],
        package,
        now,
    )?;
    save_plan(conn, &input.workspace_id, &plan, package, now)?;

    let first_concept_id = plan.queue[first_idx].concept_id.clone();
    let session_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE id = ?1)",
            [&input.session_id],
            |r| r.get(0),
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let new_rev: u32 = if session_exists {
        let curr_rev: i64 = conn
            .query_row(
                "SELECT COALESCE(session_revision, 1) FROM sessions WHERE id = ?1",
                [&input.session_id],
                |r| r.get(0),
            )
            .unwrap_or(1);
        let nr = (curr_rev + 1) as u32;
        conn.execute(
            "UPDATE sessions
             SET status = 'active', paused_at = NULL, current_attempt_id = ?2,
                 plan_id = ?3, plan_revision = ?4, session_revision = ?5,
                 problem_index = ?6, problem_count = ?7, last_activity_at = ?8
             WHERE id = ?1",
            params![
                input.session_id,
                attempt.id,
                plan.id,
                plan.revision as i64,
                nr as i64,
                (first_idx + 1) as i64,
                plan.queue.len() as i64,
                now,
            ],
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
        nr
    } else {
        conn.execute(
            "INSERT INTO sessions (
                id, workspace_id, concept_id, status, intent_activity, intent_detail,
                intent_target_minutes, resume_summary, elapsed_minutes, started_at,
                current_attempt_id, problem_index, problem_count, last_activity_at,
                plan_id, plan_revision, session_revision
             ) VALUES (?1, ?2, ?3, 'active', 'Practising', ?4, ?5, 'Adaptive practice session in progress.', 0, ?6, ?7, ?8, ?9, ?6, ?10, ?11, 1)",
            params![
                input.session_id,
                input.workspace_id,
                first_concept_id,
                format!("{} mode", plan.difficulty.as_str()),
                plan.target_ms.map(|ms| ms / 60_000),
                now,
                attempt.id,
                (first_idx + 1) as i64,
                plan.queue.len() as i64,
                plan.id,
                plan.revision as i64,
            ],
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
        1
    };

    let timing = load_or_init_timing(
        conn,
        &input.workspace_id,
        &input.session_id,
        plan.target_ms,
        input.elapsed_delta_ms,
        input.attempt_elapsed_delta_ms,
        Some(true),
        false,
        now,
    )?;

    Ok(SessionActionReceipt {
        operation_id: input.operation_id.clone(),
        session_id: input.session_id.clone(),
        revision: new_rev,
        event_ids: Vec::new(),
        attempt: Some(attempt),
        mastery_changes: Vec::new(),
        bloom_changes: Vec::new(),
        pattern_changes: Vec::new(),
        plan,
        timing,
        summary: unavailable_summary(),
    })
}

fn handle_resume_or_pause_action(
    conn: &Connection,
    package: &KnowledgePackage,
    input: &SessionActionInput,
    is_resume: bool,
    now: &str,
) -> Result<SessionActionReceipt, PracticeError> {
    let (mut plan, rev, _curr_att, summary_json) =
        load_or_create_session_plan(conn, &input.workspace_id, &input.session_id, package, now)?;
    let new_rev = rev + 1;

    if is_resume {
        conn.execute(
            "UPDATE sessions SET status = 'active', paused_at = NULL, session_revision = ?2, last_activity_at = ?3 WHERE id = ?1",
            params![input.session_id, new_rev as i64, now],
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
    } else {
        conn.execute(
            "UPDATE sessions SET status = 'paused', paused_at = ?3, session_revision = ?2, last_activity_at = ?3 WHERE id = ?1",
            params![input.session_id, new_rev as i64, now],
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
    }

    let active_idx = plan
        .queue
        .iter()
        .position(|s| s.state == "current")
        .or_else(|| plan.queue.iter().position(|s| s.state == "pending"));

    let attempt = if let Some(idx) = active_idx {
        let difficulty = plan.difficulty;
        let pid = plan.id.clone();
        Some(ensure_attempt_for_slot(
            conn,
            &input.workspace_id,
            &input.session_id,
            &pid,
            difficulty,
            &mut plan.queue[idx],
            package,
            now,
        )?)
    } else {
        None
    };

    let timing = load_or_init_timing(
        conn,
        &input.workspace_id,
        &input.session_id,
        plan.target_ms,
        input.elapsed_delta_ms,
        input.attempt_elapsed_delta_ms,
        Some(is_resume),
        false,
        now,
    )?;

    Ok(SessionActionReceipt {
        operation_id: input.operation_id.clone(),
        session_id: input.session_id.clone(),
        revision: new_rev,
        event_ids: Vec::new(),
        attempt,
        mastery_changes: Vec::new(),
        bloom_changes: Vec::new(),
        pattern_changes: Vec::new(),
        plan,
        timing,
        summary: parse_saved_summary(&input.workspace_id, package, summary_json, now),
    })
}

fn handle_save_working_action(
    conn: &Connection,
    package: &KnowledgePackage,
    input: &SessionActionInput,
    now: &str,
) -> Result<SessionActionReceipt, PracticeError> {
    let (plan, rev, curr_att, summary_json) =
        load_or_create_session_plan(conn, &input.workspace_id, &input.session_id, package, now)?;
    let attempt_id = input
        .payload
        .get("attemptId")
        .or_else(|| input.payload.get("attempt_id"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .or(curr_att)
        .ok_or_else(|| PracticeError::Storage("invalid-input: missing attemptId".to_owned()))?;

    let mut state =
        load_attempt_state(conn, &input.workspace_id, &attempt_id)?.ok_or_else(|| {
            PracticeError::AttemptNotFound {
                attempt_id: attempt_id.clone(),
            }
        })?;

    let draft_rev = input
        .payload
        .get("draftRevision")
        .or_else(|| input.payload.get("draft_revision"))
        .and_then(|v| v.as_u64())
        .unwrap_or((state.draft_revision + 1) as u64) as u32;

    if draft_rev >= state.draft_revision {
        state.draft_revision = draft_rev;
        if let Some(ans) = input
            .payload
            .get("answerText")
            .or_else(|| input.payload.get("answer_text"))
            .and_then(|v| v.as_str())
        {
            state.answer_text = ans.to_owned();
        }
        if let Some(work) = input
            .payload
            .get("workingText")
            .or_else(|| input.payload.get("working_text"))
            .and_then(|v| v.as_str())
        {
            state.working_text = work.to_owned();
        }
        save_attempt_state(conn, &input.workspace_id, &state, now)?;
    }

    let new_rev = rev + 1;
    conn.execute(
        "UPDATE sessions SET session_revision = ?2, last_activity_at = ?3 WHERE id = ?1",
        params![input.session_id, new_rev as i64, now],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let timing = load_or_init_timing(
        conn,
        &input.workspace_id,
        &input.session_id,
        plan.target_ms,
        input.elapsed_delta_ms,
        input.attempt_elapsed_delta_ms,
        None,
        false,
        now,
    )?;

    Ok(SessionActionReceipt {
        operation_id: input.operation_id.clone(),
        session_id: input.session_id.clone(),
        revision: new_rev,
        event_ids: Vec::new(),
        attempt: Some(state),
        mastery_changes: Vec::new(),
        bloom_changes: Vec::new(),
        pattern_changes: Vec::new(),
        plan,
        timing,
        summary: parse_saved_summary(&input.workspace_id, package, summary_json, now),
    })
}

fn handle_hint_action(
    conn: &Connection,
    package: &KnowledgePackage,
    input: &SessionActionInput,
    now: &str,
) -> Result<SessionActionReceipt, PracticeError> {
    let (plan, rev, curr_att, summary_json) =
        load_or_create_session_plan(conn, &input.workspace_id, &input.session_id, package, now)?;
    let attempt_id = input
        .payload
        .get("attemptId")
        .or_else(|| input.payload.get("attempt_id"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .or(curr_att)
        .ok_or_else(|| PracticeError::Storage("invalid-input: missing attemptId".to_owned()))?;

    let mut state =
        load_attempt_state(conn, &input.workspace_id, &attempt_id)?.ok_or_else(|| {
            PracticeError::AttemptNotFound {
                attempt_id: attempt_id.clone(),
            }
        })?;
    let (instance, _) = load_attempt_secret_instance(conn, &input.workspace_id, &attempt_id)?
        .ok_or_else(|| PracticeError::AttemptNotFound {
            attempt_id: attempt_id.clone(),
        })?;

    if state.hints_revealed >= state.hint_budget || state.hints_revealed >= state.hints_total {
        return Err(PracticeError::Storage(
            "budget-exhausted: hint budget exhausted for this question".to_owned(),
        ));
    }

    state.hints_revealed += 1;
    let revealed: Vec<String> = instance
        .hints
        .iter()
        .take(state.hints_revealed as usize)
        .cloned()
        .collect();
    state.revealed_hints = Some(revealed);
    save_attempt_state(conn, &input.workspace_id, &state, now)?;

    let new_rev = rev + 1;
    conn.execute(
        "UPDATE sessions SET session_revision = ?2, last_activity_at = ?3 WHERE id = ?1",
        params![input.session_id, new_rev as i64, now],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let timing = load_or_init_timing(
        conn,
        &input.workspace_id,
        &input.session_id,
        plan.target_ms,
        input.elapsed_delta_ms,
        input.attempt_elapsed_delta_ms,
        None,
        false,
        now,
    )?;

    Ok(SessionActionReceipt {
        operation_id: input.operation_id.clone(),
        session_id: input.session_id.clone(),
        revision: new_rev,
        event_ids: Vec::new(),
        attempt: Some(state),
        mastery_changes: Vec::new(),
        bloom_changes: Vec::new(),
        pattern_changes: Vec::new(),
        plan,
        timing,
        summary: parse_saved_summary(&input.workspace_id, package, summary_json, now),
    })
}

fn format_canonical_solution(sol: &ResolvedSolution) -> String {
    match sol {
        ResolvedSolution::Symbolic(s) => s.clone(),
        ResolvedSolution::Numeric(n) => format!("{n}"),
    }
}

fn handle_reveal_solution_action(
    conn: &Connection,
    package: &KnowledgePackage,
    input: &SessionActionInput,
    now: &str,
) -> Result<SessionActionReceipt, PracticeError> {
    let (mut plan, rev, curr_att, summary_json) =
        load_or_create_session_plan(conn, &input.workspace_id, &input.session_id, package, now)?;
    let attempt_id = input
        .payload
        .get("attemptId")
        .or_else(|| input.payload.get("attempt_id"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .or(curr_att)
        .ok_or_else(|| PracticeError::Storage("invalid-input: missing attemptId".to_owned()))?;

    let mut state =
        load_attempt_state(conn, &input.workspace_id, &attempt_id)?.ok_or_else(|| {
            PracticeError::AttemptNotFound {
                attempt_id: attempt_id.clone(),
            }
        })?;
    let (instance, family_id) =
        load_attempt_secret_instance(conn, &input.workspace_id, &attempt_id)?.ok_or_else(|| {
            PracticeError::AttemptNotFound {
                attempt_id: attempt_id.clone(),
            }
        })?;

    let family = package
        .problem_families
        .iter()
        .find(|f| f.id.as_str() == family_id)
        .unwrap_or_else(|| &package.problem_families[0]);

    state.hints_revealed = state.hints_total;
    state.revealed_hints = Some(instance.hints.clone());
    state.canonical_solution_display =
        Some(format_canonical_solution(&instance.canonical_solution));
    state.worked_solution = Some(family.solution_structure.clone());
    state.status = "solved".to_owned();

    let mut event_ids = Vec::new();
    let mut mastery_changes = Vec::new();

    if let Some(slot_idx) = plan.queue.iter().position(|s| s.id == state.slot_id) {
        if plan.queue[slot_idx].state == "current" {
            plan.queue[slot_idx].state = "miss".to_owned();
        }
        // Record solution-revealed observation (0 credit if no prior assessable check)
        let prior_assessable: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM practice_observations WHERE attempt_id = ?1 AND assessable = 1",
                [&attempt_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if prior_assessable == 0 {
            let obs_id = format!("obs-{}", Uuid::new_v4());
            let stamp = make_stamp(
                &input.workspace_id,
                "practice.session",
                package,
                Vec::new(),
                now,
            );
            let stamp_json =
                serde_json::to_string(&stamp).map_err(|e| PracticeError::Storage(e.to_string()))?;
            let obj_json = serde_json::to_string(&plan.queue[slot_idx].objective_ids)
                .map_err(|e| PracticeError::Storage(e.to_string()))?;
            conn.execute(
                "INSERT INTO practice_observations (
                    id, workspace_id, session_id, attempt_id, submission_id, slot_id,
                    concept_id, objective_ids_json, family_id, seed, bloom_level, difficulty,
                    outcome, assessable, raw_credit, hint_cost, effective_credit, hints_used,
                    verification_id, attribution_id, policy_id, policy_revision, content_digest,
                    observed_at, stamp_json
                 ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'solution-revealed', 1, 0.0, 0.0, 0.0, ?12, NULL, NULL, ?13, ?14, ?15, ?16, ?17)",
                params![
                    obs_id,
                    input.workspace_id,
                    input.session_id,
                    attempt_id,
                    state.slot_id,
                    plan.queue[slot_idx].concept_id,
                    obj_json,
                    family_id,
                    plan.queue[slot_idx].seed,
                    state.bloom_target,
                    plan.difficulty.as_str(),
                    state.hints_revealed as i64,
                    DEFAULT_POLICY_ID,
                    DEFAULT_POLICY_REVISION as i64,
                    package_content_ref(package).digest,
                    now,
                    stamp_json,
                ],
            )
            .map_err(|e| PracticeError::Storage(e.to_string()))?;
            event_ids.push(obs_id.clone());

            let m_snap = update_concept_mastery(
                conn,
                &input.workspace_id,
                &plan.queue[slot_idx].concept_id,
                &plan.queue[slot_idx].objective_ids,
                &obs_id,
                0.0,
                package,
                now,
            )?;
            mastery_changes.push(m_snap);
        }
    }

    save_attempt_state(conn, &input.workspace_id, &state, now)?;
    save_plan(conn, &input.workspace_id, &plan, package, now)?;

    let new_rev = rev + 1;
    conn.execute(
        "UPDATE sessions SET session_revision = ?2, last_activity_at = ?3 WHERE id = ?1",
        params![input.session_id, new_rev as i64, now],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let timing = load_or_init_timing(
        conn,
        &input.workspace_id,
        &input.session_id,
        plan.target_ms,
        input.elapsed_delta_ms,
        input.attempt_elapsed_delta_ms,
        None,
        false,
        now,
    )?;

    Ok(SessionActionReceipt {
        operation_id: input.operation_id.clone(),
        session_id: input.session_id.clone(),
        revision: new_rev,
        event_ids,
        attempt: Some(state),
        mastery_changes,
        bloom_changes: Vec::new(),
        pattern_changes: Vec::new(),
        plan,
        timing,
        summary: parse_saved_summary(&input.workspace_id, package, summary_json, now),
    })
}

fn update_concept_mastery(
    conn: &Connection,
    workspace_id: &str,
    concept_id: &str,
    objective_ids: &[String],
    observation_id: &str,
    effective_credit: f64,
    package: &KnowledgePackage,
    now: &str,
) -> Result<Available<MasterySnapshot>, PracticeError> {
    let prev_row: Option<(String, f64, i64, String)> = conn
        .query_row(
            "SELECT id, raw_score, trial_count, last_observed_at
             FROM practice_mastery_snapshots
             WHERE workspace_id = ?1 AND subject_kind = 'concept' AND subject_id = ?2
             ORDER BY last_observed_at DESC, rowid DESC LIMIT 1",
            params![workspace_id, concept_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let now_ms = parse_rfc3339_ms(now).unwrap_or(0);
    let (prev_id, prev_score, new_score, trial_count, decay_detail) =
        if let Some((pid, raw_prev, trials, last_obs)) = prev_row {
            let elapsed_ms = parse_rfc3339_ms(&last_obs)
                .map(|t| (now_ms - t).max(0))
                .unwrap_or(0);
            let decayed_prior = apply_decay(raw_prev, elapsed_ms);
            let decay_amt = (raw_prev - decayed_prior).max(0.0);
            let decay = if decay_amt > 1e-6 {
                Some(DecayDetail {
                    from_snapshot_id: pid.clone(),
                    from_score: raw_prev,
                    elapsed_ms,
                    amount: decay_amt,
                })
            } else {
                None
            };
            let next = ((1.0 - TRIAL_WEIGHT) * decayed_prior + TRIAL_WEIGHT * effective_credit)
                .clamp(0.0, 1.0);
            (
                Some(pid),
                Some(decayed_prior),
                next,
                (trials + 1) as u32,
                decay,
            )
        } else {
            let next = (TRIAL_WEIGHT * effective_credit).clamp(0.0, 1.0);
            (None, None, next, 1, None)
        };

    let delta = prev_score.map(|p| new_score - p);
    let named = named_mastery_state(new_score, trial_count).to_owned();
    let snap_id = format!("msnap-{}", Uuid::new_v4());

    let snapshot = MasterySnapshot {
        id: snap_id.clone(),
        concept_id: concept_id.to_owned(),
        objective_ids: objective_ids.to_vec(),
        score: new_score,
        previous_snapshot_id: prev_id,
        previous_score: prev_score,
        delta,
        named_state: named.clone(),
        as_of: now.to_owned(),
        last_practiced_at: now.to_owned(),
        decay: decay_detail,
    };

    let stamp = make_stamp(
        workspace_id,
        "practice.evidence",
        package,
        vec![observation_id.to_owned()],
        now,
    );
    let snap_json =
        serde_json::to_string(&snapshot).map_err(|e| PracticeError::Storage(e.to_string()))?;
    let stamp_json =
        serde_json::to_string(&stamp).map_err(|e| PracticeError::Storage(e.to_string()))?;

    conn.execute(
        "INSERT INTO practice_mastery_snapshots (
            id, workspace_id, subject_kind, subject_id, observation_id,
            raw_score, decayed_score, named_state, trial_count, last_observed_at,
            half_life_ms, snapshot_json, stamp_json
         ) VALUES (?1, ?2, 'concept', ?3, ?4, ?5, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            snap_id,
            workspace_id,
            concept_id,
            observation_id,
            new_score,
            named,
            trial_count as i64,
            now,
            HALF_LIFE_MS,
            snap_json,
            stamp_json,
        ],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    Ok(Available::Available {
        value: snapshot,
        stamp,
    })
}

fn update_bloom_if_qualified(
    conn: &Connection,
    workspace_id: &str,
    concept_id: &str,
    objective_ids: &[String],
    bloom_target: Option<&str>,
    observation_id: &str,
    package: &KnowledgePackage,
    now: &str,
) -> Result<Option<Available<BloomEvidence>>, PracticeError> {
    let level = bloom_target.unwrap_or("apply");
    let qualifying_attempts: Vec<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT attempt_id FROM practice_observations
                 WHERE workspace_id = ?1 AND concept_id = ?2 AND outcome = 'correct' AND hints_used = 0
                 ORDER BY observed_at DESC LIMIT 5",
            )
            .map_err(|e| PracticeError::Storage(e.to_string()))?;
        let rows: Vec<String> = stmt
            .query_map(params![workspace_id, concept_id], |r| r.get::<_, String>(0))
            .map_err(|e| PracticeError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        rows
    };

    if qualifying_attempts.len() < 2 {
        return Ok(None);
    }

    let prev_level: Option<String> = conn
        .query_row(
            "SELECT highest_demonstrated FROM practice_bloom_snapshots
             WHERE workspace_id = ?1 AND concept_id = ?2
             ORDER BY recorded_at DESC LIMIT 1",
            params![workspace_id, concept_id],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| PracticeError::Storage(e.to_string()))?
        .flatten();

    let bloom = BloomEvidence {
        demonstrated_level: level.to_owned(),
        objective_ids: objective_ids.to_vec(),
        qualifying_attempt_ids: qualifying_attempts,
        previous_level: prev_level,
    };

    let stamp = make_stamp(
        workspace_id,
        "practice.evidence",
        package,
        vec![observation_id.to_owned()],
        now,
    );
    let b_id = format!("bsnap-{}", Uuid::new_v4());
    let b_json =
        serde_json::to_string(&bloom).map_err(|e| PracticeError::Storage(e.to_string()))?;
    let st_json =
        serde_json::to_string(&stamp).map_err(|e| PracticeError::Storage(e.to_string()))?;

    conn.execute(
        "INSERT INTO practice_bloom_snapshots (
            id, workspace_id, concept_id, observation_id, highest_demonstrated,
            snapshot_json, stamp_json, recorded_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            b_id,
            workspace_id,
            concept_id,
            observation_id,
            level,
            b_json,
            st_json,
            now
        ],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    Ok(Some(Available::Available {
        value: bloom,
        stamp,
    }))
}

fn handle_check_action(
    conn: &Connection,
    package: &KnowledgePackage,
    input: &SessionActionInput,
    precomputed_verification: Option<VerifyV2Result>,
    now: &str,
) -> Result<SessionActionReceipt, PracticeError> {
    let (mut plan, rev, curr_att, summary_json) =
        load_or_create_session_plan(conn, &input.workspace_id, &input.session_id, package, now)?;
    let attempt_id = input
        .payload
        .get("attemptId")
        .or_else(|| input.payload.get("attempt_id"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .or(curr_att)
        .ok_or_else(|| PracticeError::Storage("invalid-input: missing attemptId".to_owned()))?;

    let mut state =
        load_attempt_state(conn, &input.workspace_id, &attempt_id)?.ok_or_else(|| {
            PracticeError::AttemptNotFound {
                attempt_id: attempt_id.clone(),
            }
        })?;

    if state.status != "open" {
        return Err(PracticeError::Storage(format!(
            "attempt-closed: attempt {attempt_id} is already {}",
            state.status
        )));
    }

    let (instance, family_id) =
        load_attempt_secret_instance(conn, &input.workspace_id, &attempt_id)?.ok_or_else(|| {
            PracticeError::AttemptNotFound {
                attempt_id: attempt_id.clone(),
            }
        })?;
    let family = package
        .problem_families
        .iter()
        .find(|f| f.id.as_str() == family_id)
        .unwrap_or_else(|| &package.problem_families[0]);

    let resp_val = input.payload.get("response").cloned().unwrap_or_else(
        || json!({ "responseType": "symbolic-expression", "value": state.answer_text }),
    );

    let answer_str = resp_val
        .get("value")
        .map(|v| {
            if let Some(s) = v.as_str() {
                s.to_owned()
            } else {
                v.to_string()
            }
        })
        .unwrap_or_else(|| state.answer_text.clone());
    state.answer_text = answer_str.clone();

    let verification = if let Some(v) = precomputed_verification {
        v
    } else {
        let canonical_val = match &instance.canonical_solution {
            ResolvedSolution::Symbolic(s) => json!(s),
            ResolvedSolution::Numeric(n) => json!(n),
        };
        let req = VerifyV2Request {
            workspace_id: input.workspace_id.clone(),
            attempt_id: attempt_id.clone(),
            response: json!(answer_str),
            canonical_solution: canonical_val,
            response_type: match family.response_type {
                crate::knowledge::ResponseType::SymbolicExpression => {
                    "symbolic-expression".to_owned()
                }
                crate::knowledge::ResponseType::Numeric => "numeric".to_owned(),
            },
            restrictions: state.restrictions.clone(),
            content_ref: ContentRefInput {
                package_id: package.id.as_str().to_owned(),
                package_version: package.version.to_string(),
                entity_id: family.id.as_str().to_owned(),
                schema_version: package.schema_version,
                digest: package_content_ref(package).digest,
            },
            objective_ids: family
                .objective_ids
                .iter()
                .map(|o| o.as_str().to_owned())
                .collect(),
        };
        verify_v2(req).map_err(|e| PracticeError::VerificationFailed(e.to_string()))?
    };

    let verdict_str = match verification.verdict {
        VerifyVerdict::Correct => "correct",
        VerifyVerdict::Incorrect => "incorrect",
        VerifyVerdict::Partial => "partial",
        VerifyVerdict::Unverifiable => "unverifiable",
    };
    let method_str = match verification.method {
        crate::capabilities::math_verify::VerifyMethod::NumericTolerance => "numeric-tolerance",
        crate::capabilities::math_verify::VerifyMethod::ConstantExpressionNumeric => {
            "constant-expression-numeric"
        }
    };

    let verification_dto = VerificationDto {
        id: verification.id.clone(),
        verdict: verdict_str.to_owned(),
        method: method_str.to_owned(),
        checker_version: verification.checker_version.clone(),
        absolute_tolerance: verification.absolute_tolerance,
        relative_tolerance: verification.relative_tolerance,
        canonical_value: verification.canonical_value,
        student_value: verification.student_value,
        restrictions: verification.restrictions.clone(),
        parse_error: verification.parse_error.clone(),
        limitations: verification.limitations.clone(),
        objective_results: verification
            .objective_results
            .iter()
            .map(|o| VerificationObjectiveDto {
                objective_id: o.objective_id.clone(),
                verdict: match o.verdict {
                    VerifyVerdict::Correct => "correct".to_owned(),
                    VerifyVerdict::Incorrect => "incorrect".to_owned(),
                    VerifyVerdict::Partial => "partial".to_owned(),
                    VerifyVerdict::Unverifiable => "unverifiable".to_owned(),
                },
                evidence: o.evidence.clone(),
            })
            .collect(),
        samples: None,
        terms: None,
    };

    let submission_id = format!("sub-{}", Uuid::new_v4());
    let is_correct = verification.verdict == VerifyVerdict::Correct;
    conn.execute(
        "INSERT INTO practice_submissions (id, attempt_id, response_json, correct, submitted_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            submission_id,
            attempt_id,
            resp_val.to_string(),
            i64::from(is_correct),
            now,
        ],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let ver_stamp = make_stamp(&input.workspace_id, "math.verify", package, Vec::new(), now);
    let ver_json = serde_json::to_string(&verification_dto)
        .map_err(|e| PracticeError::Storage(e.to_string()))?;
    let ver_stamp_json =
        serde_json::to_string(&ver_stamp).map_err(|e| PracticeError::Storage(e.to_string()))?;

    conn.execute(
        "INSERT INTO practice_verifications (
            id, workspace_id, attempt_id, submission_id, verdict, method,
            checker_version, verification_json, stamp_json, recorded_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            verification.id,
            input.workspace_id,
            attempt_id,
            submission_id,
            verdict_str,
            method_str,
            verification.checker_version,
            ver_json,
            ver_stamp_json,
            now,
        ],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    state.submission_ids.push(submission_id.clone());
    state.last_feedback_id = Some(verification.id.clone());
    state.last_verification = Some(verification_dto);

    if is_correct {
        state.status = "solved".to_owned();
        state.canonical_solution_display =
            Some(format_canonical_solution(&instance.canonical_solution));
        state.worked_solution = Some(family.solution_structure.clone());
    }

    let prior_assessable_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM practice_observations WHERE attempt_id = ?1 AND assessable = 1",
            [&attempt_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let is_assessable = matches!(
        verification.verdict,
        VerifyVerdict::Correct | VerifyVerdict::Incorrect | VerifyVerdict::Partial
    ) && prior_assessable_count == 0;

    let raw_credit = match verification.verdict {
        VerifyVerdict::Correct => Some(1.0),
        VerifyVerdict::Partial => Some(PARTIAL_CREDIT),
        VerifyVerdict::Incorrect => Some(0.0),
        VerifyVerdict::Unverifiable => None,
    };
    let hint_cost = (state.hints_revealed as f64) * plan.difficulty.cost_per_hint();
    let effective_credit = raw_credit.map(|rc| (rc - hint_cost).max(0.0));

    let slot_idx = plan
        .queue
        .iter()
        .position(|s| s.id == state.slot_id)
        .unwrap_or(0);
    let slot_concept_id = plan.queue[slot_idx].concept_id.clone();
    let slot_objective_ids = plan.queue[slot_idx].objective_ids.clone();
    let slot_seed = plan.queue[slot_idx].seed.clone();

    if is_correct {
        plan.queue[slot_idx].state = "correct".to_owned();
    } else if verification.verdict == VerifyVerdict::Partial {
        plan.queue[slot_idx].state = "partial".to_owned();
    }

    let obs_id = format!("obs-{}", Uuid::new_v4());
    let obs_stamp = make_stamp(
        &input.workspace_id,
        "practice.session",
        package,
        vec![verification.id.clone()],
        now,
    );
    let obs_stamp_json =
        serde_json::to_string(&obs_stamp).map_err(|e| PracticeError::Storage(e.to_string()))?;
    let obj_json = serde_json::to_string(&slot_objective_ids)
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    conn.execute(
        "INSERT INTO practice_observations (
            id, workspace_id, session_id, attempt_id, submission_id, slot_id,
            concept_id, objective_ids_json, family_id, seed, bloom_level, difficulty,
            outcome, assessable, raw_credit, hint_cost, effective_credit, hints_used,
            verification_id, attribution_id, policy_id, policy_revision, content_digest,
            observed_at, stamp_json
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, NULL, ?20, ?21, ?22, ?23, ?24)",
        params![
            obs_id,
            input.workspace_id,
            input.session_id,
            attempt_id,
            submission_id,
            state.slot_id,
            slot_concept_id,
            obj_json,
            family_id,
            slot_seed,
            state.bloom_target,
            plan.difficulty.as_str(),
            verdict_str,
            i64::from(is_assessable),
            raw_credit,
            hint_cost,
            effective_credit,
            state.hints_revealed as i64,
            verification.id,
            DEFAULT_POLICY_ID,
            DEFAULT_POLICY_REVISION as i64,
            package_content_ref(package).digest,
            now,
            obs_stamp_json,
        ],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let event_ids = vec![verification.id.clone(), obs_id.clone()];
    let mut mastery_changes = Vec::new();
    let mut bloom_changes = Vec::new();
    let mut pattern_changes = Vec::new();

    if is_assessable {
        if let Some(eff) = effective_credit {
            let m_snap = update_concept_mastery(
                conn,
                &input.workspace_id,
                &slot_concept_id,
                &slot_objective_ids,
                &obs_id,
                eff,
                package,
                now,
            )?;
            mastery_changes.push(m_snap);
        }

        if is_correct && state.hints_revealed == 0 {
            if let Some(b_snap) = update_bloom_if_qualified(
                conn,
                &input.workspace_id,
                &slot_concept_id,
                &slot_objective_ids,
                state.bloom_target.as_deref(),
                &obs_id,
                package,
                now,
            )? {
                bloom_changes.push(b_snap);
            }
        }
    }

    // Attribution & P4 pattern tracking if package defines opportunities or if incorrect check matches known misconception heuristic
    let pat_snaps = evaluate_and_record_attributions(
        conn,
        &input.workspace_id,
        &input.session_id,
        &attempt_id,
        &obs_id,
        &slot_concept_id,
        family,
        &instance,
        &verification,
        &answer_str,
        state.hints_revealed,
        package,
        now,
    )?;

    for ps in pat_snaps {
        if let Available::Available { ref value, .. } = ps {
            // P6: At most 1 pending slot replacement per session on confirmed observed pattern
            if value.status == "Recurring"
                || (verification.verdict == VerifyVerdict::Incorrect && value.occurrences > 0)
            {
                maybe_apply_p6_queue_replacement(
                    &mut plan,
                    &value.pattern_id,
                    &value.name,
                    &obs_id,
                );
            }
        }
        pattern_changes.push(ps);
    }

    save_attempt_state(conn, &input.workspace_id, &state, now)?;
    save_plan(conn, &input.workspace_id, &plan, package, now)?;

    let new_rev = rev + 1;
    conn.execute(
        "UPDATE sessions SET session_revision = ?2, plan_revision = ?3, last_activity_at = ?4 WHERE id = ?1",
        params![
            input.session_id,
            new_rev as i64,
            plan.revision as i64,
            now
        ],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let timing = load_or_init_timing(
        conn,
        &input.workspace_id,
        &input.session_id,
        plan.target_ms,
        input.elapsed_delta_ms,
        input.attempt_elapsed_delta_ms,
        None,
        false,
        now,
    )?;

    Ok(SessionActionReceipt {
        operation_id: input.operation_id.clone(),
        session_id: input.session_id.clone(),
        revision: new_rev,
        event_ids,
        attempt: Some(state),
        mastery_changes,
        bloom_changes,
        pattern_changes,
        plan,
        timing,
        summary: parse_saved_summary(&input.workspace_id, package, summary_json, now),
    })
}

#[allow(clippy::too_many_arguments)]
fn evaluate_and_record_attributions(
    conn: &Connection,
    workspace_id: &str,
    session_id: &str,
    attempt_id: &str,
    observation_id: &str,
    concept_id: &str,
    family: &ProblemFamily,
    instance: &ProblemInstance,
    verification: &VerifyV2Result,
    answer_str: &str,
    hints_used: u32,
    package: &KnowledgePackage,
    now: &str,
) -> Result<Vec<Available<PatternSnapshot>>, PracticeError> {
    // Check if an attribution already exists for this attempt to obey "at most one opportunity per attempt/pattern"
    let existing_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM practice_attributions WHERE workspace_id = ?1 AND attribution_json LIKE ?2",
            params![workspace_id, format!("%\"attemptId\":\"{attempt_id}\"%")],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if existing_count > 0 {
        return Ok(Vec::new());
    }

    // Build candidate pattern list from package opportunities or built-in cylindrical shell radius/prefactor detector
    struct CandidateEval {
        pattern_id: String,
        pattern_name: String,
        opportunity_id: String,
        detector_id: String,
        observed: bool,
        clean: bool,
        explanation: String,
    }

    let mut evals = Vec::new();
    let coeff = instance
        .resolved_parameters
        .get("coeff")
        .copied()
        .unwrap_or(3.0);
    let b = instance
        .resolved_parameters
        .get("b")
        .copied()
        .unwrap_or(2.0);
    // Missing 2*pi prefactor value: (c*b^3/3 - b^4/4)
    let without_two_pi = coeff * b.powi(3) / 3.0 - b.powi(4) / 4.0;
    // Missing factor of r(x) = x (integrating 2*pi*(c*x - x^2) instead of 2*pi*x*(c*x - x^2))
    let without_radius_x = 2.0 * std::f64::consts::PI * (coeff * b.powi(2) / 2.0 - b.powi(3) / 3.0);

    let student_val = verification.student_value;
    let is_missing_two_pi = student_val
        .map(|sv| (sv - without_two_pi).abs() <= 1e-6_f64.max(1e-6 * without_two_pi.abs()))
        .unwrap_or(false);
    let is_missing_radius_x = student_val
        .map(|sv| (sv - without_radius_x).abs() <= 1e-6_f64.max(1e-6 * without_radius_x.abs()))
        .unwrap_or(false);

    let opps = family.opportunities.as_deref().unwrap_or(&[]);
    if !opps.is_empty() {
        for opp in opps {
            let pat = package.patterns.iter().find(|p| p.id == opp.pattern_id);
            let pat_name = pat
                .map(|p| p.name.clone())
                .unwrap_or_else(|| opp.pattern_id.as_str().to_owned());
            let det_id = pat
                .map(|p| p.detector.id.as_str().to_owned())
                .unwrap_or_else(|| "det.package_pattern".to_owned());
            let observed = verification.verdict == VerifyVerdict::Incorrect
                && (is_missing_two_pi
                    || is_missing_radius_x
                    || opp.pattern_id.as_str().contains("shell"));
            let clean = verification.verdict == VerifyVerdict::Correct && hints_used == 0;
            evals.push(CandidateEval {
                pattern_id: opp.pattern_id.as_str().to_owned(),
                pattern_name: pat_name,
                opportunity_id: opp.id.as_str().to_owned(),
                detector_id: det_id,
                observed,
                clean,
                explanation: if observed {
                    format!(
                        "Observed misconception pattern {} on check.",
                        opp.pattern_id.as_str()
                    )
                } else if clean {
                    "Solved cleanly without hints on eligible opportunity.".to_owned()
                } else {
                    "Outcome inconclusive for pattern detector.".to_owned()
                },
            });
        }
    } else if is_missing_two_pi
        || is_missing_radius_x
        || verification.verdict == VerifyVerdict::Correct
    {
        let (pat_id, pat_name, det_id, obs, expl) = if is_missing_two_pi {
            (
                "pattern.shell_missing_2pi_prefactor",
                "Omitting 2π circumference prefactor in shell integral",
                "det.shell_missing_2pi",
                true,
                "Answer matches ∫ x·f(x) dx without the 2π circumference factor.".to_owned(),
            )
        } else if is_missing_radius_x {
            (
                "pattern.shell_missing_radius_factor",
                "Omitting shell radius factor r(x) = x in integrand",
                "det.shell_missing_radius",
                true,
                "Answer matches 2π ∫ f(x) dx without multiplying by shell radius r(x) = x."
                    .to_owned(),
            )
        } else {
            (
                "pattern.shell_missing_radius_factor",
                "Omitting shell radius factor r(x) = x in integrand",
                "det.shell_missing_radius",
                false,
                "Solved cylindrical shell volume with both 2π and radius r(x) = x.".to_owned(),
            )
        };
        let clean = verification.verdict == VerifyVerdict::Correct && hints_used == 0;
        evals.push(CandidateEval {
            pattern_id: pat_id.to_owned(),
            pattern_name: pat_name.to_owned(),
            opportunity_id: format!("opp.{}", family.id.as_str()),
            detector_id: det_id.to_owned(),
            observed: obs,
            clean,
            explanation: expl,
        });
    }

    let mut out = Vec::new();
    for ev in evals {
        let req_clean = load_pattern_required_clean(conn, workspace_id, &ev.pattern_id);
        let outcome = if ev.observed {
            "observed"
        } else if ev.clean {
            "clean"
        } else {
            "unknown"
        };
        let db_kind = if ev.observed {
            "confirmed"
        } else if ev.clean {
            "none"
        } else {
            "unknown"
        };

        let attr_id = format!("attr-{}", Uuid::new_v4());
        let attr = Attribution {
            id: attr_id.clone(),
            attempt_id: attempt_id.to_owned(),
            pattern_id: ev.pattern_id.clone(),
            rule_revision: 1,
            opportunity_id: ev.opportunity_id,
            eligible: true,
            outcome: outcome.to_owned(),
            evidence_ids: vec![observation_id.to_owned()],
            detector_id: Some(ev.detector_id.clone()),
            detector_version: Some(1),
            explanation: ev.explanation,
            recorded_at: Some(now.to_owned()),
            answer_text: Some(answer_str.to_owned()),
            concept_id: Some(concept_id.to_owned()),
            session_id: Some(session_id.to_owned()),
        };

        let stamp = make_stamp(
            workspace_id,
            "practice.patterns",
            package,
            vec![observation_id.to_owned()],
            now,
        );
        let attr_json =
            serde_json::to_string(&attr).map_err(|e| PracticeError::Storage(e.to_string()))?;
        let stamp_json =
            serde_json::to_string(&stamp).map_err(|e| PracticeError::Storage(e.to_string()))?;

        conn.execute(
            "INSERT INTO practice_attributions (
                id, workspace_id, observation_id, kind, pattern_id, detector_id,
                attribution_json, stamp_json, recorded_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                attr_id,
                workspace_id,
                observation_id,
                db_kind,
                ev.pattern_id,
                ev.detector_id,
                attr_json,
                stamp_json,
                now,
            ],
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

        let existing_snap: Option<PatternSnapshot> = conn
            .query_row(
                "SELECT snapshot_json FROM practice_pattern_snapshots WHERE workspace_id = ?1 AND pattern_id = ?2",
                params![workspace_id, ev.pattern_id],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| PracticeError::Storage(e.to_string()))?
            .and_then(|s| serde_json::from_str(&s).ok());

        // Only persist pattern snapshot if already tracked or if observed at least once
        if existing_snap.is_some() || ev.observed {
            let mut snap = existing_snap.unwrap_or_else(|| PatternSnapshot {
                id: format!("patsnap-{workspace_id}-{}", ev.pattern_id),
                pattern_id: ev.pattern_id.clone(),
                name: ev.pattern_name.clone(),
                concept_ids: Some(vec![concept_id.to_owned()]),
                status: "Unknown".to_owned(),
                opportunities: 0,
                occurrences: 0,
                unknown_count: 0,
                clean_streak: 0,
                first_seen_at: Some(now.to_owned()),
                last_seen_at: None,
                recent_attribution_ids: Vec::new(),
                rule_revision: 1,
                resolution: PatternResolutionInfo {
                    required_clean: req_clean,
                    eligible_clean: 0,
                    explanation: String::new(),
                },
            });

            snap.opportunities += 1;
            if ev.observed {
                snap.occurrences += 1;
                snap.clean_streak = 0;
                snap.last_seen_at = Some(now.to_owned());
                if snap.first_seen_at.is_none() {
                    snap.first_seen_at = Some(now.to_owned());
                }
            } else if ev.clean {
                snap.clean_streak += 1;
            } else {
                snap.unknown_count += 1;
                // P4: unknown_resets = false
            }

            snap.recent_attribution_ids.insert(0, attr_id);
            snap.recent_attribution_ids.truncate(10);
            snap.resolution.required_clean = req_clean;
            snap.resolution.eligible_clean = snap.clean_streak;
            snap.resolution.explanation = format!(
                "{} / {} consecutive clean unassisted opportunities.",
                snap.clean_streak, req_clean
            );

            snap.status = if snap.occurrences > 0 && snap.clean_streak >= req_clean {
                "Resolved".to_owned()
            } else if snap.occurrences > 0 && snap.clean_streak > 0 {
                "Improving".to_owned()
            } else if snap.occurrences > 0 {
                "Recurring".to_owned()
            } else {
                "Unknown".to_owned()
            };

            let db_status = match snap.status.as_str() {
                "Recurring" => "active",
                "Improving" => "fading",
                "Resolved" => "resolved",
                _ => "watching",
            };

            let snap_json =
                serde_json::to_string(&snap).map_err(|e| PracticeError::Storage(e.to_string()))?;

            conn.execute(
                "INSERT INTO practice_pattern_snapshots (
                    id, workspace_id, pattern_id, status, confirmed_count, candidate_count,
                    clean_streak, required_clean, last_observed_at, snapshot_json, stamp_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(workspace_id, pattern_id) DO UPDATE SET
                    status = excluded.status,
                    confirmed_count = excluded.confirmed_count,
                    clean_streak = excluded.clean_streak,
                    required_clean = excluded.required_clean,
                    last_observed_at = excluded.last_observed_at,
                    snapshot_json = excluded.snapshot_json,
                    stamp_json = excluded.stamp_json,
                    updated_at = excluded.updated_at",
                params![
                    snap.id,
                    workspace_id,
                    ev.pattern_id,
                    db_status,
                    snap.occurrences as i64,
                    snap.clean_streak as i64,
                    req_clean as i64,
                    snap.last_seen_at,
                    snap_json,
                    stamp_json,
                    now,
                ],
            )
            .map_err(|e| PracticeError::Storage(e.to_string()))?;

            out.push(Available::Available { value: snap, stamp });
        }
    }

    Ok(out)
}

fn maybe_apply_p6_queue_replacement(
    plan: &mut Plan,
    pattern_id: &str,
    pattern_name: &str,
    observation_id: &str,
) {
    // P6: replacement_limit = 1 per session
    if !plan.changes.is_empty() {
        return;
    }
    if let Some(pending_idx) = plan.queue.iter().position(|s| s.state == "pending") {
        let old_slot = plan.queue[pending_idx].clone();
        let new_slot_id = format!("{}-rep", old_slot.id);
        let new_seed = (old_slot.seed.parse::<u64>().unwrap_or(100) + 19).to_string();
        let reason = format!(
            "Queue adjusted after observing \"{}\": replaced pending slot {} with a targeted follow-up check.",
            pattern_name, old_slot.id
        );
        plan.queue[pending_idx] = PlanQueueSlot {
            id: new_slot_id.clone(),
            phase_id: old_slot.phase_id,
            concept_id: old_slot.concept_id,
            objective_ids: old_slot.objective_ids,
            family_id: old_slot.family_id,
            seed: new_seed,
            attempt_id: None,
            state: "pending".to_owned(),
            reason: reason.clone(),
            evidence_ids: vec![observation_id.to_owned()],
            replaces_id: Some(old_slot.id.clone()),
        };
        plan.changes.push(PlanChange {
            event_id: observation_id.to_owned(),
            removed_slot_id: old_slot.id,
            added_slot_id: new_slot_id,
            reason,
        });
        plan.revision += 1;
        let _ = pattern_id;
    }
}

fn handle_skip_or_advance_action(
    conn: &Connection,
    package: &KnowledgePackage,
    input: &SessionActionInput,
    is_skip: bool,
    now: &str,
) -> Result<SessionActionReceipt, PracticeError> {
    let (mut plan, rev, curr_att, summary_json) =
        load_or_create_session_plan(conn, &input.workspace_id, &input.session_id, package, now)?;

    let active_idx = plan.queue.iter().position(|s| {
        s.state == "current"
            || (curr_att.is_some() && s.attempt_id.as_deref() == curr_att.as_deref())
    });

    if let Some(idx) = active_idx {
        if let Some(att_id) = plan.queue[idx].attempt_id.clone() {
            if let Some(mut state) = load_attempt_state(conn, &input.workspace_id, &att_id)? {
                if is_skip && state.status == "open" {
                    state.status = "skipped".to_owned();
                    save_attempt_state(conn, &input.workspace_id, &state, now)?;
                    plan.queue[idx].state = "skipped".to_owned();

                    // Record skipped observation (assessable = 0)
                    let obs_id = format!("obs-{}", Uuid::new_v4());
                    let stamp = make_stamp(
                        &input.workspace_id,
                        "practice.session",
                        package,
                        Vec::new(),
                        now,
                    );
                    let stamp_json = serde_json::to_string(&stamp)
                        .map_err(|e| PracticeError::Storage(e.to_string()))?;
                    let obj_json = serde_json::to_string(&plan.queue[idx].objective_ids)
                        .map_err(|e| PracticeError::Storage(e.to_string()))?;
                    conn.execute(
                        "INSERT INTO practice_observations (
                            id, workspace_id, session_id, attempt_id, submission_id, slot_id,
                            concept_id, objective_ids_json, family_id, seed, bloom_level, difficulty,
                            outcome, assessable, raw_credit, hint_cost, effective_credit, hints_used,
                            verification_id, attribution_id, policy_id, policy_revision, content_digest,
                            observed_at, stamp_json
                         ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'skipped', 0, NULL, 0.0, NULL, ?12, NULL, NULL, ?13, ?14, ?15, ?16, ?17)",
                        params![
                            obs_id,
                            input.workspace_id,
                            input.session_id,
                            att_id,
                            state.slot_id,
                            plan.queue[idx].concept_id,
                            obj_json,
                            plan.queue[idx].family_id,
                            plan.queue[idx].seed,
                            state.bloom_target,
                            plan.difficulty.as_str(),
                            state.hints_revealed as i64,
                            DEFAULT_POLICY_ID,
                            DEFAULT_POLICY_REVISION as i64,
                            package_content_ref(package).digest,
                            now,
                            stamp_json,
                        ],
                    )
                    .map_err(|e| PracticeError::Storage(e.to_string()))?;
                } else if plan.queue[idx].state == "current" {
                    plan.queue[idx].state = if state.status == "solved" {
                        "correct".to_owned()
                    } else {
                        "skipped".to_owned()
                    };
                }
            }
        }
    }

    let next_pending_idx = plan.queue.iter().position(|s| s.state == "pending");
    if let Some(next_idx) = next_pending_idx {
        let difficulty = plan.difficulty;
        let pid = plan.id.clone();
        let next_state = ensure_attempt_for_slot(
            conn,
            &input.workspace_id,
            &input.session_id,
            &pid,
            difficulty,
            &mut plan.queue[next_idx],
            package,
            now,
        )?;
        save_plan(conn, &input.workspace_id, &plan, package, now)?;

        let new_rev = rev + 1;
        conn.execute(
            "UPDATE sessions
             SET current_attempt_id = ?2, problem_index = ?3, problem_count = ?4,
                 session_revision = ?5, last_activity_at = ?6
             WHERE id = ?1",
            params![
                input.session_id,
                next_state.id,
                (next_idx + 1) as i64,
                plan.queue.len() as i64,
                new_rev as i64,
                now,
            ],
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

        let timing = load_or_init_timing(
            conn,
            &input.workspace_id,
            &input.session_id,
            plan.target_ms,
            input.elapsed_delta_ms,
            input.attempt_elapsed_delta_ms,
            None,
            true,
            now,
        )?;

        Ok(SessionActionReceipt {
            operation_id: input.operation_id.clone(),
            session_id: input.session_id.clone(),
            revision: new_rev,
            event_ids: Vec::new(),
            attempt: Some(next_state),
            mastery_changes: Vec::new(),
            bloom_changes: Vec::new(),
            pattern_changes: Vec::new(),
            plan,
            timing,
            summary: parse_saved_summary(&input.workspace_id, package, summary_json, now),
        })
    } else {
        // No more pending questions -> freeze session summary
        save_plan(conn, &input.workspace_id, &plan, package, now)?;
        handle_end_action(conn, package, input, now)
    }
}

fn handle_edit_plan_action(
    conn: &Connection,
    package: &KnowledgePackage,
    input: &SessionActionInput,
    now: &str,
) -> Result<SessionActionReceipt, PracticeError> {
    let (mut plan, rev, curr_att, summary_json) =
        load_or_create_session_plan(conn, &input.workspace_id, &input.session_id, package, now)?;

    if let Some(diff_str) = input.payload.get("difficulty").and_then(|v| v.as_str()) {
        plan.difficulty = match diff_str {
            "Push" => PracticeDifficulty::Push,
            "Exam" => PracticeDifficulty::Exam,
            _ => PracticeDifficulty::Steady,
        };
    }
    if let Some(t_ms) = input
        .payload
        .get("targetMs")
        .or_else(|| input.payload.get("target_ms"))
        .and_then(|v| v.as_i64())
    {
        plan.target_ms = Some(t_ms.clamp(600_000, 2_700_000));
    }

    plan.revision += 1;
    save_plan(conn, &input.workspace_id, &plan, package, now)?;

    let new_rev = rev + 1;
    conn.execute(
        "UPDATE sessions SET plan_revision = ?2, session_revision = ?3, last_activity_at = ?4 WHERE id = ?1",
        params![input.session_id, plan.revision as i64, new_rev as i64, now],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let attempt = if let Some(ref att_id) = curr_att {
        load_attempt_state(conn, &input.workspace_id, att_id)?
    } else {
        None
    };

    let timing = load_or_init_timing(
        conn,
        &input.workspace_id,
        &input.session_id,
        plan.target_ms,
        input.elapsed_delta_ms,
        input.attempt_elapsed_delta_ms,
        None,
        false,
        now,
    )?;

    Ok(SessionActionReceipt {
        operation_id: input.operation_id.clone(),
        session_id: input.session_id.clone(),
        revision: new_rev,
        event_ids: Vec::new(),
        attempt,
        mastery_changes: Vec::new(),
        bloom_changes: Vec::new(),
        pattern_changes: Vec::new(),
        plan,
        timing,
        summary: parse_saved_summary(&input.workspace_id, package, summary_json, now),
    })
}

fn handle_end_action(
    conn: &Connection,
    package: &KnowledgePackage,
    input: &SessionActionInput,
    now: &str,
) -> Result<SessionActionReceipt, PracticeError> {
    let (mut plan, rev, _curr_att, existing_summary_json) =
        load_or_create_session_plan(conn, &input.workspace_id, &input.session_id, package, now)?;

    let timing = load_or_init_timing(
        conn,
        &input.workspace_id,
        &input.session_id,
        plan.target_ms,
        input.elapsed_delta_ms,
        input.attempt_elapsed_delta_ms,
        Some(false),
        false,
        now,
    )?;

    // End is idempotent: if already completed and summary_json exists, return frozen summary
    if let Some(s_json) = existing_summary_json {
        if let Ok(summary) = serde_json::from_str::<SessionSummary>(&s_json) {
            return Ok(SessionActionReceipt {
                operation_id: input.operation_id.clone(),
                session_id: input.session_id.clone(),
                revision: rev,
                event_ids: Vec::new(),
                attempt: None,
                mastery_changes: summary.mastery_changes.clone(),
                bloom_changes: Vec::new(),
                pattern_changes: Vec::new(),
                plan,
                timing,
                summary: Available::Available {
                    value: summary,
                    stamp: make_stamp(
                        &input.workspace_id,
                        "practice.session",
                        package,
                        Vec::new(),
                        now,
                    ),
                },
            });
        }
    }

    let mut correct = 0u32;
    let mut assessed_total = 0u32;
    let mut attempted_total = 0u32;
    let mut skipped_total = 0u32;
    let mut unverifiable_total = 0u32;
    let mut hints_used = 0u32;
    let mut hint_budget_total = 0u32;
    let mut question_rows = Vec::new();
    let mut difficulty_history = Vec::new();

    for slot in &mut plan.queue {
        difficulty_history.push(DifficultyHistoryEntry {
            slot_id: slot.id.clone(),
            difficulty: plan.difficulty,
            policy_ref: default_policy_ref(),
        });

        let Some(ref att_id) = slot.attempt_id else {
            continue;
        };
        let Some(att_state) = load_attempt_state(conn, &input.workspace_id, att_id)? else {
            continue;
        };

        hints_used += att_state.hints_revealed;
        hint_budget_total += att_state.hint_budget;

        let res_str = match slot.state.as_str() {
            "correct" => {
                correct += 1;
                assessed_total += 1;
                attempted_total += 1;
                "correct"
            }
            "partial" => {
                assessed_total += 1;
                attempted_total += 1;
                "partial"
            }
            "miss" => {
                assessed_total += 1;
                attempted_total += 1;
                "incorrect"
            }
            "skipped" => {
                skipped_total += 1;
                "skipped"
            }
            _ => {
                if !att_state.submission_ids.is_empty() {
                    attempted_total += 1;
                    if let Some(ref ver) = att_state.last_verification {
                        if ver.verdict == "unverifiable" {
                            unverifiable_total += 1;
                            "unverifiable"
                        } else {
                            assessed_total += 1;
                            slot.state = "miss".to_owned();
                            "incorrect"
                        }
                    } else {
                        assessed_total += 1;
                        "incorrect"
                    }
                } else {
                    skipped_total += 1;
                    slot.state = "skipped".to_owned();
                    "skipped"
                }
            }
        };

        let phase_name = plan
            .phases
            .iter()
            .find(|p| p.id == slot.phase_id)
            .map(|p| p.name.clone());

        question_rows.push(SessionQuestionRow {
            slot_id: slot.id.clone(),
            attempt_id: att_id.clone(),
            phase_id: slot.phase_id.clone(),
            phase_name,
            concept_id: Some(slot.concept_id.clone()),
            prompt: att_state.prompt,
            result: res_str.to_owned(),
            feedback_id: att_state.last_feedback_id,
            before_snapshot_id: None,
            after_snapshot_id: None,
            before_score: None,
            after_score: None,
        });
    }

    save_plan(conn, &input.workspace_id, &plan, package, now)?;

    let mastery_changes: Vec<Available<MasterySnapshot>> = {
        let mut stmt = conn
            .prepare(
                "SELECT m.snapshot_json, m.stamp_json
                 FROM practice_mastery_snapshots m
                 JOIN practice_observations o ON o.id = m.observation_id
                 WHERE o.session_id = ?1
                 ORDER BY m.last_observed_at DESC",
            )
            .map_err(|e| PracticeError::Storage(e.to_string()))?;
        let mut seen_concepts = HashSet::new();
        let mut list = Vec::new();
        let rows = stmt
            .query_map([&input.session_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(|e| PracticeError::Storage(e.to_string()))?;
        for r in rows.flatten() {
            if let (Ok(snap), Ok(st)) = (
                serde_json::from_str::<MasterySnapshot>(&r.0),
                serde_json::from_str::<Stamp>(&r.1),
            ) {
                if seen_concepts.insert(snap.concept_id.clone()) {
                    list.push(Available::Available {
                        value: snap,
                        stamp: st,
                    });
                }
            }
        }
        list
    };

    let repeated_pattern_ids: Vec<String> = {
        let mut stmt = conn
            .prepare(
                "SELECT DISTINCT pattern_id FROM practice_attributions
                 WHERE workspace_id = ?1 AND kind = 'confirmed' AND pattern_id IS NOT NULL",
            )
            .map_err(|e| PracticeError::Storage(e.to_string()))?;
        let rows: Vec<String> = stmt
            .query_map([&input.workspace_id], |r| r.get::<_, String>(0))
            .map_err(|e| PracticeError::Storage(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        rows
    };

    let summary = SessionSummary {
        session_id: input.session_id.clone(),
        completed_at: now.to_owned(),
        timing: timing.clone(),
        difficulty_history,
        correct,
        assessed_total,
        attempted_total,
        skipped_total,
        unverifiable_total,
        hints_used,
        hint_budget_total: Some(hint_budget_total),
        mastery_changes: mastery_changes.clone(),
        repeated_pattern_ids,
        question_rows,
        next_plan: Available::Available {
            value: plan.clone(),
            stamp: make_stamp(
                &input.workspace_id,
                "practice.plan",
                package,
                Vec::new(),
                now,
            ),
        },
    };

    let summary_json =
        serde_json::to_string(&summary).map_err(|e| PracticeError::Storage(e.to_string()))?;
    let new_rev = rev + 1;
    let elapsed_mins = match &timing {
        Available::Available { value, .. } => (value.elapsed_ms / 60_000).max(0),
        _ => 0,
    };

    conn.execute(
        "UPDATE sessions
         SET status = 'completed', paused_at = NULL, completed_at = ?2,
             summary_json = ?3, session_revision = ?4, elapsed_minutes = ?5,
             last_activity_at = ?2
         WHERE id = ?1",
        params![
            input.session_id,
            now,
            summary_json,
            new_rev as i64,
            elapsed_mins,
        ],
    )
    .map_err(|e| PracticeError::Storage(e.to_string()))?;

    Ok(SessionActionReceipt {
        operation_id: input.operation_id.clone(),
        session_id: input.session_id.clone(),
        revision: new_rev,
        event_ids: Vec::new(),
        attempt: None,
        mastery_changes,
        bloom_changes: Vec::new(),
        pattern_changes: Vec::new(),
        plan,
        timing,
        summary: Available::Available {
            value: summary,
            stamp: make_stamp(
                &input.workspace_id,
                "practice.session",
                package,
                Vec::new(),
                now,
            ),
        },
    })
}

pub fn list_session_history(
    store: &PracticeStore,
    package: &KnowledgePackage,
    workspace_id: &str,
) -> Result<Vec<SessionHistoryEntry>, PracticeError> {
    let conn = store.lock_connection()?;
    let now = now_iso();

    let mut stmt = conn
        .prepare(
            "SELECT id, status, started_at, completed_at, summary_json, plan_id
             FROM sessions
             WHERE workspace_id = ?1
             ORDER BY COALESCE(completed_at, last_activity_at, started_at) DESC, rowid DESC",
        )
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let rows = stmt
        .query_map([workspace_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<String>>(5)?,
            ))
        })
        .map_err(|e| PracticeError::Storage(e.to_string()))?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| PracticeError::Storage(e.to_string()))?;

    let mut entries = Vec::with_capacity(rows.len());
    for (session_id, status, started_at, completed_at, summary_json, plan_id) in rows {
        let difficulty = plan_id
            .and_then(|pid| load_plan_by_id(&conn, workspace_id, &pid).ok())
            .map(|p| p.difficulty)
            .or(Some(PracticeDifficulty::Steady));

        let summary = if let Some(s_json) = summary_json {
            if let Ok(sum) = serde_json::from_str::<SessionSummary>(&s_json) {
                Available::Available {
                    value: sum,
                    stamp: make_stamp(workspace_id, "practice.session", package, Vec::new(), &now),
                }
            } else {
                Available::Unavailable {
                    reason: UnavailableReason::LegacyOnly,
                    last_known: None,
                }
            }
        } else {
            Available::Unavailable {
                reason: UnavailableReason::NoEvidence,
                last_known: None,
            }
        };

        entries.push(SessionHistoryEntry {
            session_id,
            workspace_id: workspace_id.to_owned(),
            status,
            started_at,
            completed_at,
            difficulty,
            summary,
        });
    }

    Ok(entries)
}
