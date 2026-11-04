pub mod engine;
mod error;
mod provider;
mod store;
mod types;

pub use error::PracticeError;
pub use provider::PracticeProvider;
pub use store::PracticeStore;
pub use types::{
    AttemptState, AttemptStatus, Attribution, Available, BloomEvidence, ConceptDescriptor,
    ConceptEvidence, ConceptsRequest, ConceptsResponse, ContentRef, CreatePlanInput, DecayDetail,
    DescribeRequest, DescribeResponse, DifficultyHistoryEntry, EditPatternRuleInput,
    EvaluateRequest, EvaluateResponse, EvidenceReadRequest, GenerateRequest, GenerateResponse,
    HintRequest, HintResponse, LastKnownValue, MasterySnapshot, PatternDetail,
    PatternDetailReadRequest, PatternListResponse, PatternResolutionInfo, PatternSessionBucket,
    PatternSnapshot, PatternsReadRequest, Plan, PlanChange, PlanGoalSnapshot, PlanPhase,
    PlanQueueSlot, PolicyRef, PracticeDifficulty, PrerequisiteReadiness, ProviderStampRef,
    Readiness, ResponseValue, SessionActionInput, SessionActionReceipt, SessionHistoryEntry,
    SessionQuestionRow, SessionSummary, SessionTiming, Stamp, StartRequest, StartResponse,
    UnavailableReason, VerificationDto, VerificationObjectiveDto,
};

/// The embedded first-party manifest for this module.
pub const MANIFEST_TOML: &str = include_str!("module.toml");

#[cfg(test)]
mod tests;
