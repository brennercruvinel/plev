//! an intent read as a change of the surface shown (task-70 phase 2):
//! the second grammar of the rules, beside the one of surfaces
//! (`words.rs`), with the same folding and the same two languages.
//!
//! four changes: **less** ("too much", "cluttered", "simpler"; "demais",
//! "poluído", "mais simples"), **compare** only with a word of difficulty
//! ("can't compare", "hard to compare"; "não consigo comparar"; a plain
//! "compare the original with the cuts" stays the comparison of views of
//! the surface grammar), **bigger** ("bigger numbers", "can't see them";
//! "números maiores", "não enxergo") and **undo** ("undo"; "desfazer").
//! every other word is a filler of either language, a word of the
//! surface grammar taken as a hint, or a word left for the app, which
//! chooses the target.

use crate::words::{fold, split};

/// the changes the rules read, as a person reads them.
pub const CHANGES: &str = "too much (or cluttered, simpler), can't compare, bigger (or can't \
                           see) and undo (demais, mais simples, não consigo comparar, maior, \
                           não enxergo, desfazer)";

const LESS: &[&str] = &[
    "cluttered",
    "clutter",
    "busy",
    "crowded",
    "noisy",
    "overwhelming",
    "simpler",
    "simplify",
    "less",
    "fewer",
    "demais",
    "poluido",
    "poluida",
    "carregado",
    "carregada",
    "confuso",
    "confusa",
    "simples",
    "simplificar",
    "simplifica",
    "menos",
    "enxuto",
    "enxugar",
];
/// "too much" and "too many": a quantity the word before makes too much.
const MUCH: &[&str] = &["much", "many"];
const TOO: &[&str] = &["too"];
const COMPARE: &[&str] = &[
    "compare",
    "comparing",
    "comparison",
    "comparar",
    "compara",
    "comparacao",
];
const BIGGER: &[&str] = &[
    "bigger", "larger", "big", "large", "grow", "enlarge", "increase", "maior", "maiores",
    "grande", "grandes", "aumentar", "aumente", "aumenta", "ampliar", "amplie",
];
/// seeing or reading, which a difficulty makes a request for bigger.
const SEE: &[&str] = &[
    "see", "read", "enxergar", "enxergo", "enxerga", "ler", "leio", "ver", "vejo",
];
/// a difficulty: what makes "compare" and "see" a request.
const HARD: &[&str] = &[
    "cannot",
    "cant",
    "t",
    "not",
    "hard",
    "difficult",
    "difficulty",
    "unable",
    "struggle",
    "nao",
    "dificil",
    "dificuldade",
];
const UNDO: &[&str] = &[
    "undo", "revert", "desfazer", "desfaz", "desfaca", "voltar", "volta", "reverter", "reverta",
];
/// the words a change skips beside the surface grammar's fillers.
const FILLER: &str = "here there s this these those them they information info stuff thing \
    things it all can could may please again way look looks make get now bit little lot very so \
    more have has having am m ve \
    aqui ai ali tem tenho ha isso informacao informacoes coisa coisas tudo consigo posso pode da \
    muito muita mais bem pouco agora tanta tanto estou eles elas";
/// the words of the surface grammar a change reads as a hint of its
/// target, never as a surface: "the numbers bigger", "a lista demais".
const HINTS: &str = "number numero count counter contagem contador stat statistic estatistica \
    total item itens list lista video image imagem frame quadro value valor text texto";

fn is(word: &str, list: &[&str]) -> bool {
    list.iter().any(|w| fold(w) == word)
}

fn listed(word: &str, list: &str) -> bool {
    list.split_whitespace().any(|w| fold(w) == word)
}

/// an intent read as a change.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Asked {
    pub less: bool,
    pub compare: bool,
    pub bigger: bool,
    pub undo: bool,
    /// the words left for the app: (as written, folded).
    pub words: Vec<(String, String)>,
}

impl Asked {
    /// the intent asks for a change the rules make.
    pub fn is_change(&self) -> bool {
        self.less || self.compare || self.bigger || self.undo
    }
}

/// reads `intent` as a change of the surface shown.
pub(crate) fn read(intent: &str) -> Asked {
    let mut asked = Asked::default();
    let written: Vec<&str> = split(intent).collect();
    let folded: Vec<String> = written.iter().map(|w| fold(w)).collect();
    let hard = folded.iter().any(|w| is(w, HARD));
    let mut compare = None;
    for (at, (w, word)) in written.iter().zip(&folded).enumerate() {
        let before = at.checked_sub(1).and_then(|b| folded.get(b));
        if is(word, LESS) || (is(word, MUCH) && before.is_some_and(|b| is(b, TOO))) {
            asked.less = true;
        } else if is(word, COMPARE) {
            compare.get_or_insert(((*w).to_owned(), word.clone()));
        } else if is(word, BIGGER) || (is(word, SEE) && hard) {
            asked.bigger = true;
        } else if is(word, UNDO) {
            asked.undo = true;
        } else if is(word, HARD) || is(word, TOO) || is(word, MUCH) || is(word, SEE) {
            // a difficulty, or a quantity or a sight nothing made a request
        } else if listed(word, FILLER) || listed(word, HINTS) || crate::words::is_filler(word) {
            // skipped
        } else {
            asked.words.push(((*w).to_owned(), word.clone()));
        }
    }
    match compare {
        Some(_) if hard => asked.compare = true,
        // a comparison without a difficulty is a surface of views
        Some(word) => asked.words.push(word),
        None => {}
    }
    asked
}

/// true when `intent` reads as a change of the surface shown, whatever
/// the app's words: what a page asks before it asks a model for
/// operations instead of a surface.
pub fn reads_as_change(intent: &str) -> bool {
    read(intent).is_change()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(a: &Asked) -> Vec<&str> {
        a.words.iter().map(|(_, w)| w.as_str()).collect()
    }

    #[test]
    fn the_changes_read_in_both_languages() {
        for less in [
            "too much information",
            "there's too much here",
            "it looks cluttered",
            "simpler, please",
            "tem informação demais",
            "está poluído",
            "mais simples",
        ] {
            let a = read(less);
            assert!(
                a.less && !a.compare && !a.bigger && !a.undo,
                "{less}: {a:?}"
            );
            assert!(a.words.is_empty(), "{less}: {a:?}");
        }
        for compare in [
            "I can't compare these",
            "I cannot compare them",
            "hard to compare",
            "não consigo comparar",
            "difícil de comparar isso",
        ] {
            let a = read(compare);
            assert!(a.compare && !a.less, "{compare}: {a:?}");
        }
        for bigger in [
            "bigger numbers",
            "números maiores",
            "I can't see the numbers",
            "tenho dificuldade para enxergar esses números",
            "aumentar os números",
        ] {
            let a = read(bigger);
            assert!(a.bigger && !a.compare, "{bigger}: {a:?}");
            assert!(a.words.is_empty(), "{bigger}: {a:?}");
        }
        for undo in ["undo", "desfazer", "undo that", "desfaz isso"] {
            assert!(read(undo).undo, "{undo}");
        }
        let durations = read("bigger durations");
        assert!(durations.bigger);
        assert_eq!(words(&durations), ["duration"]);
    }

    /// what is not a change: a surface of the other grammar, a plain
    /// comparison of views, a word the rules do not read.
    #[test]
    fn a_surface_is_not_a_change() {
        for surface in [
            "items",
            "two columns",
            "compact",
            "only what is active",
            "tudo",
        ] {
            assert!(!reads_as_change(surface), "{surface}");
        }
        let views = read("compare the original with the cuts");
        assert!(!views.is_change());
        assert_eq!(words(&views), ["original", "cut", "compare"]);
        assert!(!reads_as_change("I can see them"));
    }
}
