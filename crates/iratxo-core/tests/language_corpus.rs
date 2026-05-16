//! Corpus-based ground truth for language detection.
//!
//! Per language: ~10 short authentic sentences spanning casual greetings,
//! commerce/support vocabulary, and longer compound utterances. Each sentence
//! must be classified to its source language. We require *full* accuracy —
//! short sentences are where heuristics are weakest, so a missed case here
//! signals a real regression rather than a noisy edge.
//!
//! The corpus deliberately mixes registers so any single signal (e.g. a
//! particular function word, a unique diacritic) is not load-bearing.

use iratxo_core::text::{detect_language, Language};

struct Sample {
    lang: Language,
    text: &'static str,
}

const CORPUS: &[Sample] = &[
    // English
    Sample { lang: Language::English, text: "Hello there, how are you today?" },
    Sample { lang: Language::English, text: "I would like to cancel my subscription please." },
    Sample { lang: Language::English, text: "The customer requested a full refund last week." },
    Sample { lang: Language::English, text: "Please review our updated privacy policy and terms of service." },
    Sample { lang: Language::English, text: "We must terminate the contract before the end of the month." },
    Sample { lang: Language::English, text: "Thanks for your patience while we look into this issue." },
    Sample { lang: Language::English, text: "The team is working on a fix and will report back soon." },
    Sample { lang: Language::English, text: "Could you please send your invoice and order number?" },

    // Spanish
    Sample { lang: Language::Spanish, text: "Hola, ¿cómo estás hoy?" },
    Sample { lang: Language::Spanish, text: "Quiero cancelar mi suscripción cuanto antes." },
    Sample { lang: Language::Spanish, text: "El cliente solicitó un reembolso completo la semana pasada." },
    Sample { lang: Language::Spanish, text: "Por favor, revise nuestra política de privacidad actualizada." },
    Sample { lang: Language::Spanish, text: "Debemos rescindir el contrato antes de fin de mes." },
    Sample { lang: Language::Spanish, text: "Gracias por su paciencia mientras investigamos el problema." },
    Sample { lang: Language::Spanish, text: "¿Podría enviarnos su número de pedido y la factura?" },
    Sample { lang: Language::Spanish, text: "El equipo está trabajando en una solución urgente." },

    // Catalan
    Sample { lang: Language::Catalan, text: "Hola, com estàs avui?" },
    Sample { lang: Language::Catalan, text: "Vull cancel·lar la meva subscripció al més aviat possible." },
    Sample { lang: Language::Catalan, text: "El client va sol·licitar un reemborsament complet la setmana passada." },
    Sample { lang: Language::Catalan, text: "Si us plau, reviseu la nostra política de privacitat actualitzada." },
    Sample { lang: Language::Catalan, text: "Hem de rescindir el contracte abans de finals de mes." },
    Sample { lang: Language::Catalan, text: "Gràcies per la vostra paciència mentre investiguem el problema." },
    Sample { lang: Language::Catalan, text: "Podeu enviar-nos el número de comanda i la factura?" },

    // Basque
    Sample { lang: Language::Basque, text: "Kaixo, zer moduz zaude gaur?" },
    Sample { lang: Language::Basque, text: "Nire harpidetza bertan behera utzi nahi dut ahalik eta lasterren." },
    Sample { lang: Language::Basque, text: "Bezeroak diru itzulketa osoa eskatu zuen joan den astean." },
    Sample { lang: Language::Basque, text: "Mesedez, berrikusi gure pribatutasun politika eguneratua." },
    Sample { lang: Language::Basque, text: "Kontratua hilabete amaiera baino lehen bertan behera utzi behar dugu." },
    Sample { lang: Language::Basque, text: "Eskerrik asko zure pazientziagatik arazoa aztertzen dugun bitartean." },

    // French
    Sample { lang: Language::French, text: "Bonjour, comment allez-vous aujourd'hui?" },
    Sample { lang: Language::French, text: "Je voudrais résilier mon abonnement le plus tôt possible." },
    Sample { lang: Language::French, text: "Le client a demandé un remboursement complet la semaine dernière." },
    Sample { lang: Language::French, text: "Veuillez consulter notre politique de confidentialité mise à jour." },
    Sample { lang: Language::French, text: "Nous devons résilier le contrat avant la fin du mois." },
    Sample { lang: Language::French, text: "Merci pour votre patience pendant que nous étudions ce problème." },
    Sample { lang: Language::French, text: "Pouvez-vous nous envoyer votre numéro de commande et la facture?" },
    Sample { lang: Language::French, text: "L'équipe travaille sur une solution et reviendra vers vous bientôt." },

    // Italian
    Sample { lang: Language::Italian, text: "Buongiorno, come stai oggi?" },
    Sample { lang: Language::Italian, text: "Vorrei cancellare il mio abbonamento il prima possibile." },
    Sample { lang: Language::Italian, text: "Il cliente ha richiesto un rimborso completo la settimana scorsa." },
    Sample { lang: Language::Italian, text: "Si prega di consultare la nostra informativa sulla privacy aggiornata." },
    Sample { lang: Language::Italian, text: "Dobbiamo rescindere il contratto entro la fine del mese." },
    Sample { lang: Language::Italian, text: "Grazie per la pazienza mentre esaminiamo il problema." },
    Sample { lang: Language::Italian, text: "Può inviarci il numero d'ordine e la fattura?" },
    Sample { lang: Language::Italian, text: "Il team sta lavorando a una soluzione e tornerà presto." },

    // German
    Sample { lang: Language::German, text: "Guten Tag, wie geht es Ihnen heute?" },
    Sample { lang: Language::German, text: "Ich möchte mein Abonnement so bald wie möglich kündigen." },
    Sample { lang: Language::German, text: "Der Kunde hat letzte Woche eine vollständige Rückerstattung beantragt." },
    Sample { lang: Language::German, text: "Bitte lesen Sie unsere aktualisierte Datenschutzerklärung." },
    Sample { lang: Language::German, text: "Wir müssen den Vertrag vor Monatsende auflösen." },
    Sample { lang: Language::German, text: "Vielen Dank für Ihre Geduld, während wir das Problem prüfen." },
    Sample { lang: Language::German, text: "Könnten Sie uns Ihre Bestellnummer und die Rechnung senden?" },
    Sample { lang: Language::German, text: "Das Team arbeitet an einer Lösung und meldet sich bald zurück." },

    // Dutch
    Sample { lang: Language::Dutch, text: "Goedendag, hoe gaat het vandaag?" },
    Sample { lang: Language::Dutch, text: "Ik wil mijn abonnement zo snel mogelijk opzeggen." },
    Sample { lang: Language::Dutch, text: "De klant heeft vorige week een volledige terugbetaling gevraagd." },
    Sample { lang: Language::Dutch, text: "Raadpleeg ons bijgewerkte privacybeleid en de algemene voorwaarden." },
    Sample { lang: Language::Dutch, text: "We moeten het contract voor het einde van de maand beëindigen." },
    Sample { lang: Language::Dutch, text: "Bedankt voor je geduld terwijl we het probleem onderzoeken." },
    Sample { lang: Language::Dutch, text: "Kun je ons je bestelnummer en de factuur sturen?" },
    Sample { lang: Language::Dutch, text: "Het team werkt aan een oplossing en komt zo snel mogelijk terug." },
];

#[test]
fn every_corpus_sample_is_detected_correctly() {
    let mut failures: Vec<String> = Vec::new();
    for sample in CORPUS {
        let got = detect_language(sample.text);
        if got != sample.lang {
            failures.push(format!(
                "expected {:?}, got {:?} for: {:?}",
                sample.lang, got, sample.text
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} mis-classifications out of {} samples:\n  {}",
        failures.len(),
        CORPUS.len(),
        failures.join("\n  ")
    );
}

#[test]
fn every_language_has_at_least_six_samples() {
    // Guard against accidentally trimming the corpus during refactors. Six is
    // the floor; English/Spanish/French/etc. carry more.
    for &lang in Language::all() {
        let n = CORPUS.iter().filter(|s| s.lang == lang).count();
        assert!(n >= 6, "{:?} has only {} samples; need ≥ 6", lang, n);
    }
}
