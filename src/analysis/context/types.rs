use crate::analysis::parse::Parse;

/// Local grammatical evidence; weights are not probabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextRule {
    LeftModifierAgreement,
    RightNounAgreement,
    LeftSubjectAgreement,
    FollowingInfinitive,
    PossibleGenitiveComplement,
    PrepositionCase,
    PossibleInvertedSubject,
    GovernedInfinitive,
    ImperativeObject,
}

impl ContextRule {
    /// Fixed contribution to a candidate's contextual support.
    pub fn weight(self) -> u32 {
        match self {
            Self::LeftModifierAgreement | Self::RightNounAgreement | Self::FollowingInfinitive => 3,
            Self::LeftSubjectAgreement => 5,
            Self::PossibleGenitiveComplement | Self::PossibleInvertedSubject => 2,
            Self::PrepositionCase | Self::ImperativeObject => 4,
            Self::GovernedInfinitive => 6,
        }
    }
}

/// Support for one original candidate, without changing its Parse.score.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateSupport {
    /// Index into ContextAnalysis.candidates.
    pub index: usize,
    /// Sum of the evidence weights.
    pub support: u32,
    /// Applied rules, in evaluation order.
    pub evidence: Vec<ContextRule>,
}

/// A lemma/POS pair, possibly containing several grammatical analyses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextReading {
    pub lemma: String,
    pub pos: String,
    /// Indices into ContextAnalysis.candidates, in original order.
    pub parse_indices: Vec<usize>,
}

/// Whether local rules distinguish a single lemma/POS pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextStatus {
    NoCandidates,
    /// A single pair remains without contextual evidence.
    OnlyReading,
    /// Rules support a single pair; this does not guarantee correctness.
    SupportedReading,
    /// Several pairs have equal positive support.
    Ambiguous,
    /// Several pairs remain and no rule applies.
    NoEvidence,
}

/// A contextual decision with no implicit fallback to the baseline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextDecision {
    pub status: ContextStatus,
    /// First original candidate, or None when there are no candidates.
    pub baseline_index: Option<usize>,
    /// Present only when retained contains one lemma/POS pair.
    pub selected: Option<ContextReading>,
    /// All pairs tied for maximum support, in original order.
    pub retained: Vec<ContextReading>,
    /// Support for every original candidate.
    pub support: Vec<CandidateSupport>,
}

/// Original word analyses and a separate contextual decision for one input token.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextAnalysis {
    /// Input spelling, without normalization.
    pub token: String,
    /// Complete parse(token) output, with unchanged order and scores.
    pub candidates: Vec<Parse>,
    pub decision: ContextDecision,
}
