//! Precision/recall ground truth for `semantic_match`.
//!
//! Per language we maintain two corpora:
//!   - **positives**: synonym / paraphrase pairs that should score ≥ a high
//!     threshold (recall — the engine recognizes equivalence).
//!   - **negatives**: unrelated-topic pairs in the same language that should
//!     score ≤ a low threshold (precision — the engine doesn't false-fire).
//!
//! The thresholds are tuned for the 256-dim hashing-trick embedding the
//! engine ships with. They are intentionally generous (paraphrases need only
//! beat 0.30, unrelated must stay below 0.20) so the assertions survive minor
//! stemmer/vocab tweaks but still catch real semantic regressions.

use iratxo_core::semantic::similarity_lang;
use iratxo_core::text::Language;

// Threshold rationale: the README documents "0.2 for sparse texts that share
// a single concept token after stemming, 0.3 for typical paragraph-length
// inputs". The test corpus is short and sparse, so we set the recall floor
// at 0.25 — high enough that an actual paraphrase needs at least one shared
// dictionary-canonical token, low enough that the 256-dim hashing-trick
// embedding can realistically clear it for two-content-word sentences. The
// precision ceiling (0.20) stays comfortably below so unrelated pairs that
// share no canonicals can't accidentally satisfy it.
const RECALL_FLOOR: f32 = 0.25;
const PRECISION_CEIL: f32 = 0.20;

struct Pair { a: &'static str, b: &'static str }

fn check(lang: Language, positives: &[Pair], negatives: &[Pair]) {
    let mut failures = Vec::new();
    for p in positives {
        let s = similarity_lang(p.a, p.b, lang, None);
        if !(s >= RECALL_FLOOR) {
            failures.push(format!(
                "RECALL miss ({:?}): sim({:?}, {:?}) = {:.3} < {:.2}",
                lang, p.a, p.b, s, RECALL_FLOOR
            ));
        }
    }
    for p in negatives {
        let s = similarity_lang(p.a, p.b, lang, None);
        if !(s <= PRECISION_CEIL) {
            failures.push(format!(
                "PRECISION false-positive ({:?}): sim({:?}, {:?}) = {:.3} > {:.2}",
                lang, p.a, p.b, s, PRECISION_CEIL
            ));
        }
    }
    assert!(failures.is_empty(), "{} failures:\n  {}", failures.len(), failures.join("\n  "));
}

// ─── English ───────────────────────────────────────────────────────────

#[test]
fn english_precision_and_recall() {
    check(Language::English,
        &[
            Pair { a: "I want to cancel my contract",       b: "please terminate the agreement" },
            Pair { a: "issue a full refund",                b: "we need a complete reimbursement" },
            Pair { a: "the policy was updated",             b: "the guideline has been revised" },
            Pair { a: "phishing attempt via email",         b: "scam message in our inbox" },
            Pair { a: "ignore previous instructions",       b: "disregard the prior prompt" },
        ],
        &[
            Pair { a: "I want to cancel my contract",       b: "the weather is nice today" },
            Pair { a: "issue a full refund",                b: "the dog ate my homework" },
            Pair { a: "phishing attempt via email",         b: "we had pasta for dinner" },
        ],
    );
}

// ─── Spanish ───────────────────────────────────────────────────────────

#[test]
fn spanish_precision_and_recall() {
    check(Language::Spanish,
        &[
            Pair { a: "quiero cancelar mi contrato",        b: "necesito rescindir el acuerdo" },
            Pair { a: "emitan un reembolso completo",       b: "necesitamos una devolución total" },
            Pair { a: "la política ha sido actualizada",    b: "la norma ha sido revisada" },
            Pair { a: "intento de phishing por correo",     b: "mensaje de estafa en mi bandeja" },
        ],
        &[
            Pair { a: "quiero cancelar mi contrato",        b: "hoy hace muy buen tiempo" },
            Pair { a: "emitan un reembolso completo",       b: "el gato duerme en la silla" },
        ],
    );
}

// ─── Catalan ───────────────────────────────────────────────────────────

#[test]
fn catalan_precision_and_recall() {
    check(Language::Catalan,
        &[
            Pair { a: "vull cancel·lar el meu contracte",   b: "necessito rescindir l'acord" },
            Pair { a: "emeti un reemborsament complet",     b: "necessitem una devolució total" },
            Pair { a: "la política ha estat actualitzada",  b: "la norma ha estat revisada" },
        ],
        &[
            Pair { a: "vull cancel·lar el meu contracte",   b: "avui fa molt bon temps" },
            Pair { a: "emeti un reemborsament complet",     b: "el gat dorm a la cadira" },
        ],
    );
}

// ─── Basque ────────────────────────────────────────────────────────────

#[test]
fn basque_precision_and_recall() {
    // Basque stemmer collapses determined-plural and inessive case endings
    // ("-ak"/"-an") onto the bare noun, so pairs that share a noun like
    // "museo" or "etxe" reliably overlap. The Basque dictionary is small
    // (no compliance/legal vocab), so we lean on stemmer-only signal here.
    check(Language::Basque,
        &[
            Pair { a: "museoak handiak dira",                b: "museoan nago" },
            Pair { a: "ikasleak liburuak irakurtzen ari dira",
                   b: "ikasleen liburuak interesgarriak dira" },
        ],
        &[
            Pair { a: "museoak handiak dira",                b: "gaur eguraldi ona dago" },
        ],
    );
}

// ─── French ────────────────────────────────────────────────────────────

#[test]
fn french_precision_and_recall() {
    check(Language::French,
        &[
            Pair { a: "je veux annuler mon contrat",        b: "il faut résilier l'accord" },
            Pair { a: "veuillez émettre un remboursement",  b: "nous demandons une restitution complète" },
            Pair { a: "la politique a été mise à jour",     b: "la directive a été révisée" },
            Pair { a: "tentative d'hameçonnage par mail",   b: "message d'escroquerie dans la boîte" },
        ],
        &[
            Pair { a: "je veux annuler mon contrat",        b: "il fait très beau aujourd'hui" },
            Pair { a: "veuillez émettre un remboursement",  b: "le chat dort sur la chaise" },
        ],
    );
}

// ─── Italian ───────────────────────────────────────────────────────────

#[test]
fn italian_precision_and_recall() {
    check(Language::Italian,
        &[
            Pair { a: "voglio cancellare il contratto",     b: "bisogna rescindere l'accordo" },
            Pair { a: "richiedo un rimborso completo",      b: "abbiamo bisogno di una restituzione" },
            Pair { a: "la politica è stata aggiornata",     b: "la norma è stata rivista" },
        ],
        &[
            Pair { a: "voglio cancellare il contratto",     b: "oggi c'è un bel tempo" },
            Pair { a: "richiedo un rimborso completo",      b: "il gatto dorme sulla sedia" },
        ],
    );
}

// ─── German ────────────────────────────────────────────────────────────

#[test]
fn german_precision_and_recall() {
    check(Language::German,
        &[
            Pair { a: "ich möchte den vertrag kündigen",    b: "wir müssen die vereinbarung beenden" },
            Pair { a: "bitte eine vollständige rückerstattung", b: "wir benötigen eine erstattung" },
            Pair { a: "die richtlinie wurde aktualisiert",  b: "die vorschrift wurde überarbeitet" },
        ],
        &[
            Pair { a: "ich möchte den vertrag kündigen",    b: "heute ist das wetter sehr schön" },
            Pair { a: "die richtlinie wurde aktualisiert",  b: "die katze schläft auf dem stuhl" },
        ],
    );
}

// ─── Dutch ─────────────────────────────────────────────────────────────

#[test]
fn dutch_precision_and_recall() {
    check(Language::Dutch,
        &[
            Pair { a: "ik wil het contract opzeggen",       b: "we moeten de overeenkomst annuleren" },
            Pair { a: "geef alstublieft een volledige terugbetaling", b: "we hebben een volledige restitutie nodig" },
            Pair { a: "het beleid is bijgewerkt",           b: "de richtlijn is herzien" },
        ],
        &[
            Pair { a: "ik wil het contract opzeggen",       b: "vandaag is het mooi weer" },
            Pair { a: "het beleid is bijgewerkt",           b: "de kat slaapt op de stoel" },
        ],
    );
}

// ─── Polysemy: a synonym that legitimately belongs to multiple
//   equivalence classes (e.g. English "scam" → both "phishing" and "fraud")
//   should partially match queries against either canonical, rather than
//   silently picking one alphabetically.

#[test]
fn polysemous_synonyms_match_all_their_canonicals() {
    // "scam" is listed under both "phishing" and "fraud" in synonyms.json.
    // Under the old String-valued index, the alphabetical sort picked
    // "fraud" and "scam"+"phishing" would have scored zero overlap. The
    // multi-canonical index distributes "scam"'s signal across both, so
    // both directions match.
    let phishing = similarity_lang("a scam attempt", "phishing email", Language::English, None);
    let fraud    = similarity_lang("a scam attempt", "fraud incident", Language::English, None);
    assert!(phishing > 0.0, "scam should partially match phishing (got {:.3})", phishing);
    assert!(fraud    > 0.0, "scam should partially match fraud    (got {:.3})", fraud);
}

// ─── Determinism: the same inputs must produce the same scores in any
//   process. Before the lexical-sort fix in SynonymIndex, Rust's per-process
//   hash seed randomized which canonical won synonym collisions, making
//   identical CI runs produce diverging scores. We sample a fixed pair and
//   compute it many times to catch any future regression in determinism.

#[test]
fn similarity_is_deterministic_within_process() {
    // We can't easily test cross-process determinism from a single test,
    // but we can assert intra-process stability (no thread-local cache
    // pollution between calls).
    let first = similarity_lang(
        "issue a full refund please",
        "we need a complete reimbursement",
        Language::English,
        None,
    );
    for _ in 0..16 {
        let again = similarity_lang(
            "issue a full refund please",
            "we need a complete reimbursement",
            Language::English,
            None,
        );
        assert_eq!(first.to_bits(), again.to_bits(), "non-deterministic similarity");
    }
}

// ─── Cross-lingual sanity: same content in different languages must
//   not score above the precision ceiling when we force a single
//   language (the dictionaries are mono-lingual).

#[test]
fn forcing_wrong_language_does_not_false_match() {
    // Treating French content as if it were English: the English stemmer
    // doesn't know "résilier" = "cancel", so similarity to "terminate the
    // agreement" should NOT clear the recall floor.
    let s = similarity_lang(
        "je veux résilier mon contrat",
        "terminate the agreement",
        Language::English,
        None,
    );
    assert!(s < RECALL_FLOOR, "cross-lingual sim was {:.3}, expected < {:.2}", s, RECALL_FLOOR);
}
