//! Ground-truth stemmer behavior: equivalence classes and pinned outputs.
//!
//! The goal is *collapse* — inflected forms of the same lemma should share a
//! stem so semantic_match treats them as identical signals. Pure Snowball is
//! not lossless and some morphological pairs don't collapse (e.g. German
//! `gekündigt` doesn't reduce to the same stem as `kündigen`); those are
//! pinned as documented-known-quirks tests rather than ignored.
//!
//! Ground truth was produced by running the corresponding Snowball algorithm
//! over the listed forms (see crates/iratxo-core/examples/stem_probe.rs).

use iratxo_core::text::{stem, Language};
use std::collections::HashSet;

fn assert_collapses(lang: Language, group: &[&str], expected_stem: &str) {
    let stems: Vec<String> = group.iter().map(|w| stem(w, lang)).collect();
    let uniq: HashSet<&String> = stems.iter().collect();
    assert_eq!(
        uniq.len(),
        1,
        "{:?} group {:?} produced multiple stems {:?}",
        lang, group, stems
    );
    assert_eq!(
        stems[0], expected_stem,
        "{:?} group {:?} collapsed to {:?}, expected {:?}",
        lang, group, stems[0], expected_stem
    );
}

// ─── English ───────────────────────────────────────────────────────────

#[test]
fn english_equivalence_classes() {
    assert_collapses(Language::English, &["cancel","cancels","cancelled","cancelling"], "cancel");
    assert_collapses(Language::English, &["agreement","agreements"], "agreement");
    assert_collapses(Language::English, &["refund","refunds","refunded","refunding"], "refund");
    assert_collapses(Language::English, &["terminate","terminates","terminated","terminating","termination"], "termin");
}

// ─── Spanish ───────────────────────────────────────────────────────────

#[test]
fn spanish_equivalence_classes() {
    assert_collapses(Language::Spanish, &["cancelar","cancelado","cancelación"], "cancel");
    assert_collapses(Language::Spanish, &["contrato","contratos"], "contrat");
    assert_collapses(Language::Spanish, &["reembolso","reembolsos","reembolsar","reembolsado"], "reembols");
    assert_collapses(Language::Spanish, &["política","políticas"], "polit");
}

// ─── Catalan (hand-rolled stemmer) ─────────────────────────────────────

#[test]
fn catalan_equivalence_classes() {
    // The Catalan stemmer folds diacritics and strips plural/feminine endings.
    let a = stem("contracte", Language::Catalan);
    let b = stem("contractes", Language::Catalan);
    assert_eq!(a, b, "contracte/contractes split: {:?} vs {:?}", a, b);

    let a = stem("política", Language::Catalan);
    let b = stem("polítiques", Language::Catalan);
    assert_eq!(a, b, "política/polítiques split: {:?} vs {:?}", a, b);

    // Diacritic-folding sanity (documented in the stemmer module).
    assert_eq!(stem("política", Language::Catalan), stem("politica", Language::Catalan));
}

// ─── Basque (hand-port of marrow) ──────────────────────────────────────

#[test]
fn basque_equivalence_classes() {
    // The marrow port strips the determined-plural `-ak` and the inessive
    // `-an`. Verify on multiple stems.
    assert_eq!(stem("museoak", Language::Basque), "museo");
    assert_eq!(stem("museoan", Language::Basque), "museo");
    assert_eq!(stem("ikasleak", Language::Basque), "ikasle");
    assert_eq!(stem("liburuak", Language::Basque), "liburu");
    // The singular determined form `-a` is *not* stripped (documented quirk
    // of the marrow algorithm — `ikaslea` keeps `-a`).
    assert_eq!(stem("ikaslea",  Language::Basque), "ikaslea");
}

// ─── French ────────────────────────────────────────────────────────────

#[test]
fn french_equivalence_classes() {
    assert_collapses(Language::French, &["politique","politiques"], "polit");
    assert_collapses(Language::French,
        &["rembourser","remboursement","remboursements","remboursé"],
        "rembours",
    );
    assert_collapses(Language::French,
        &["garantir","garantie","garanties","garantissons"],
        "garant",
    );
    assert_collapses(Language::French,
        &["utilisateur","utilisateurs","utiliser","utilisé","utilisée","utilisation"],
        "utilis",
    );
    // Partial collapse: -ation/-er/-é cluster but not first-person plural -ons.
    assert_collapses(Language::French, &["annulation","annuler","annulé","annulés"], "annul");
}

#[test]
fn french_documented_snowball_quirks() {
    // Snowball French keeps the -ons inflection separate. Pin so a stemmer
    // upgrade that changes this surfaces in CI rather than silently shifting
    // semantic match scores.
    assert_eq!(stem("annulons", Language::French), "annulon");
    // -ssion is not folded into the -mer family.
    assert_eq!(stem("suppression", Language::French), "suppress");
    assert_eq!(stem("supprimer",   Language::French), "supprim");
    // payable retains its adjectival ending.
    assert_eq!(stem("payable", Language::French), "payabl");
    assert_eq!(stem("paiement", Language::French), "pai");
}

// ─── Italian ───────────────────────────────────────────────────────────

#[test]
fn italian_equivalence_classes() {
    assert_collapses(Language::Italian,
        &["cancellare","cancellazione","cancellato","cancellati","cancellata"],
        "cancell",
    );
    assert_collapses(Language::Italian, &["politica","politiche","politici"], "polit");
    assert_collapses(Language::Italian,
        &["rimborsare","rimborso","rimborsi","rimborsata"],
        "rimbors",
    );
    assert_collapses(Language::Italian, &["utente","utenti"], "utent");
    assert_collapses(Language::Italian, &["utilizzare","utilizzato","utilizzazione"], "utilizz");
    assert_collapses(Language::Italian, &["eliminare","eliminazione","eliminato"], "elimin");
    assert_collapses(Language::Italian, &["pagare","pagamento","pagamenti"], "pag");
    assert_collapses(Language::Italian, &["supporto","supportare"], "support");
}

#[test]
fn italian_documented_snowball_quirks() {
    // The Italian Snowball stemmer treats -anza/-enza nouns (garanzia,
    // garanzie) as a distinct family from the -ire/-ito verb (garantire,
    // garantito). Recorded so the divergence is intentional, not a bug.
    assert_eq!(stem("garantire", Language::Italian), "garant");
    assert_eq!(stem("garanzia",  Language::Italian), "garanz");
}

// ─── German ────────────────────────────────────────────────────────────

#[test]
fn german_equivalence_classes() {
    assert_collapses(Language::German, &["vertrag","verträge","vertraglich"], "vertrag");
    assert_collapses(Language::German, &["richtlinie","richtlinien"], "richtlini");
    assert_collapses(Language::German, &["sicher","sicherer","sicherheit"], "sich");
}

#[test]
fn german_documented_snowball_quirks() {
    // Snowball German folds umlauts (ä→a, ü→u, ö→o), but past participles
    // with the `ge-` prefix and -t suffix don't reduce to the verb stem.
    assert_eq!(stem("kündigen",   Language::German), "kundig");
    assert_eq!(stem("kündigung",  Language::German), "kundig");
    assert_eq!(stem("kündigungen",Language::German), "kundig");
    assert_eq!(stem("gekündigt",  Language::German), "gekundigt");

    // Rückerstattung and erstatten share lemma but the prefix stays attached.
    assert_eq!(stem("rückerstattung", Language::German), "ruckerstatt");
    assert_eq!(stem("erstatten",      Language::German), "erstatt");
}

// ─── Dutch ─────────────────────────────────────────────────────────────

#[test]
fn dutch_equivalence_classes() {
    assert_collapses(Language::Dutch, &["terugbetaling","terugbetalingen","terugbetalen"], "terugbetal");
    assert_collapses(Language::Dutch, &["betalen","betaling","betalingen"], "betal");
}

#[test]
fn dutch_documented_snowball_quirks() {
    // Snowball Dutch reduces -ing nouns and -en infinitives to a shared stem,
    // but past participles with `ge-` prefix don't fold back. Pin for stability.
    assert_eq!(stem("opzeggen",  Language::Dutch), "opzegg");
    assert_eq!(stem("opzegging", Language::Dutch), "opzegg");
    assert_eq!(stem("opgezegd",  Language::Dutch), "opgezegd");

    assert_eq!(stem("gebruiker", Language::Dutch), "gebruiker");
    assert_eq!(stem("gebruiken", Language::Dutch), "gebruik");
}

// ─── Empty/short input safety ──────────────────────────────────────────

#[test]
fn empty_input_stems_to_empty() {
    for &lang in Language::all() {
        assert_eq!(stem("", lang), "", "{:?} empty input should stem to empty", lang);
    }
}
