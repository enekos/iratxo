pub mod ir;
pub mod dsl;
pub mod interpret;
pub mod semantic;
pub mod text;

pub use interpret::{evaluate, evaluate_ref, evaluate_with_metrics, EvalMetrics, EvalResult, EvalResultRef, TriggeredRule, TriggeredRuleRef};
pub use ir::{Predicate, Program, Rule, Verdict};

/// `IRTX` magic + IR version. Any change to the IR layout that is not backwards
/// compatible MUST bump this. Engines refuse unknown versions.
///
/// Version history:
///   1 — initial format.
///   2 — adds `HasUrlToDomain`, `MostlyUppercase`, `TokenEntropyAbove` variants.
///   3 — adds Catalan to `Language` (extends `LanguageIs` codes), seven new
///       `EntityKind` variants (`IpAddress`, `CreditCard`, `Iban`, `DateIso`,
///       `Hashtag`, `Mention`, `Emoji`), and twelve new heuristic predicates
///       (`WordContainsAny`, `StartsWithAny`, `EndsWithAny`, `SentenceCount`,
///       `CharCount`, `LineCount`, `DigitRatioAbove`, `PunctuationRatioAbove`,
///       `RepeatedCharRun`, `RepeatedToken`, `TypeTokenRatioBelow`,
///       `HasInvisibleChars`, `HasMixedScriptToken`, `ScriptIs`).
///   4 — boxes `Predicate::SemanticMatch` data to shrink `Predicate` enum
///       size from ~96 bytes to ~40 bytes, improving cache locality.
pub const IR_MAGIC: &[u8; 4] = b"IRTX";
pub const IR_VERSION: u16 = 4;

#[derive(Debug)]
pub enum DecodeError {
    BadMagic,
    UnsupportedVersion(u16),
    Truncated,
    Bincode(String),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::BadMagic => write!(f, "not an iratxo IR blob (missing IRTX magic)"),
            DecodeError::UnsupportedVersion(v) => write!(f, "unsupported IR version: {} (this engine speaks {})", v, IR_VERSION),
            DecodeError::Truncated => write!(f, "IR blob is truncated"),
            DecodeError::Bincode(s) => write!(f, "bincode error: {}", s),
        }
    }
}

impl std::error::Error for DecodeError {}

pub fn compile_yaml(src: &str) -> Result<Program, dsl::DslError> {
    dsl::parse(src)
}

/// Encode a program as a versioned, self-describing IR blob.
/// Layout: `IRTX` (4) | version: u16 LE (2) | bincode payload
pub fn encode(program: &Program) -> Vec<u8> {
    let payload = bincode::serialize(program).expect("serialize Program");
    let mut out = Vec::with_capacity(6 + payload.len());
    out.extend_from_slice(IR_MAGIC);
    out.extend_from_slice(&IR_VERSION.to_le_bytes());
    out.extend_from_slice(&payload);
    out
}

pub fn decode(bytes: &[u8]) -> Result<Program, DecodeError> {
    if bytes.len() < 6 { return Err(DecodeError::Truncated); }
    if &bytes[0..4] != IR_MAGIC { return Err(DecodeError::BadMagic); }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != IR_VERSION { return Err(DecodeError::UnsupportedVersion(version)); }
    let mut program: Program = bincode::deserialize(&bytes[6..])
        .map_err(|e| DecodeError::Bincode(e.to_string()))?;
    program.chained_targets = program.rules
        .iter()
        .flat_map(|r| r.then.iter().cloned())
        .collect();
    Ok(program)
}
#[cfg(test)]
mod size_tests {
    #[test]
    fn predicate_size() {
        println!("Predicate size = {}", std::mem::size_of::<crate::Predicate>());
        println!("Option<u32> size = {}", std::mem::size_of::<Option<u32>>());
    }
}
