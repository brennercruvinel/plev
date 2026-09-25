//! the intent as the rules read it.
//!
//! every word is folded (lowercase, no accent, no plural `s`) and is one
//! of three things: a word of the grammar (a part, a form, "everything",
//! "compare"),
//! a filler the grammar skips (articles, "only", "field"), or a word left
//! for the app, which must be one of the app's own words. the grammar is
//! the proposer's, so it reads english and portuguese; the app's words
//! are the app's, in its language, and nothing here translates them.

use std::collections::BTreeSet;
use std::fmt;

/// a kind of thing a surface shows, each found in the registry by its
/// structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Part {
    /// a text input of the app: the field, and a button when the input
    /// has an action with no parameter to commit it.
    Entry,
    /// a choice of the app (a tab list, a select, a radio group): one chip
    /// per option whose action takes no parameter.
    Choice,
    /// a number the app shows (an `int` or a `num` state): one stat each.
    Count,
    /// a list of the app's items (a `List` or `Set` state whose values key
    /// its node's children): the list repeated, each item with what it
    /// shows and accepts.
    List,
    /// a view of the app's media (a media or image node with a frame ref
    /// state, a text naming a media node and an instant): one image per
    /// view shown.
    Media,
}

impl Part {
    pub const ALL: [Part; 5] = [
        Part::Entry,
        Part::Choice,
        Part::Count,
        Part::List,
        Part::Media,
    ];
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Part::Entry => "an entry (add)",
            Part::Choice => "a choice (filter)",
            Part::Count => "counts",
            Part::List => "a list (items)",
            Part::Media => "a view of the media (video)",
        })
    }
}

/// how the surface is laid out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// one column: the entry, the lists, the choices, the counts, one
    /// under the other.
    Compact,
    /// two columns where the width holds them: the choices and the counts
    /// in one, the entry over the lists in the other.
    Columns,
}

/// the words of the grammar, as a person reads them.
pub const GRAMMAR: &str = "add, filter, counts (or stats), items (or list), video (or image, \
                           frame), compare, compact, two columns and everything (adicionar, \
                           filtro, contagem, itens, vídeo, comparar, compacto, duas colunas, \
                           tudo)";

const ADD: &[&str] = &[
    "add",
    "adicionar",
    "adiciona",
    "adicione",
    "new",
    "novo",
    "nova",
    "entry",
    "entrada",
];
const CHOICE: &[&str] = &["filter", "filtro", "filtrar", "filtra"];
const COUNT: &[&str] = &[
    "count",
    "counter",
    "contagem",
    "contagens",
    "contador",
    "stat",
    "stats",
    "statistic",
    "estatistica",
    "number",
    "numero",
    "total",
];
const LIST: &[&str] = &["item", "items", "itens", "list", "lista", "listar"];
const MEDIA: &[&str] = &[
    "video", "videos", "image", "imagem", "imagens", "frame", "frames", "quadro", "media", "midia",
    "picture", "preview", "previa",
];
/// "compare": every view of the media, side by side.
const COMPARE: &[&str] = &[
    "compare",
    "comparar",
    "compara",
    "comparison",
    "comparacao",
    "versus",
    "vs",
    "diff",
];
const COMPACT: &[&str] = &["compact", "compacto", "compacta", "single"];
const COLUMNS: &[&str] = &["column", "coluna"];
/// "one column" and "uma coluna" are the compact form.
const ONE: &[&str] = &["one", "uma", "um", "1", "single", "unica"];
const EVERYTHING: &[&str] = &["everything", "tudo"];
/// the words the grammar skips, english then portuguese, as written.
const FILLER: &str = "a an the and or of to in on at for with by as into what which that this \
    these those is are be it its me my i want need show give see keep make put only just please \
    field button box surface panel view screen form layout two 2 side \
    o os as um uma e ou de do da dos das no na nos nas em com para pra por pelo pela que qual \
    isso isto esse essa este esta estao sao ser eu meu minha quero preciso mostre mostrar mostra \
    ver deixe deixa so apenas somente campo botao caixa superficie painel tela formulario dois duas";

/// `word` folded: lowercase, the accents of portuguese dropped, and a
/// plural `s` dropped from a word longer than three letters.
pub(crate) fn fold(word: &str) -> String {
    let plain: String = word
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            c => c,
        })
        .collect();
    match plain.strip_suffix('s') {
        Some(stem) if plain.chars().count() > 3 => stem.to_owned(),
        _ => plain,
    }
}

/// the words of `text` as written, split at anything that is not a
/// letter or a digit.
pub(crate) fn split(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
}

/// the folded words of `text`.
pub(crate) fn folded(text: &str) -> BTreeSet<String> {
    split(text).map(fold).collect()
}

fn is(word: &str, list: &[&str]) -> bool {
    list.iter().any(|w| fold(w) == word)
}

/// true when the grammar skips `word` (folded): a filler of either
/// language, or a number word of "one column".
pub(crate) fn is_filler(word: &str) -> bool {
    FILLER.split_whitespace().any(|f| fold(f) == word) || is(word, ONE)
}

/// an intent read by the grammar.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Reading {
    /// the parts the intent names.
    pub parts: BTreeSet<Part>,
    /// "everything": every part the app has.
    pub everything: bool,
    /// "compare": every view of each media node, side by side (it names
    /// the media part too).
    pub compare: bool,
    pub form: Option<Form>,
    /// the word that asked for the entry, as written: the label of a
    /// button no other name is found for.
    pub add: Option<String>,
    /// the words left for the app: (as written, folded).
    pub words: Vec<(String, String)>,
}

impl Reading {
    /// the intent says nothing the rules read.
    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
            && !self.everything
            && !self.compare
            && self.form.is_none()
            && self.words.is_empty()
    }
}

/// reads `intent` with the grammar.
pub(crate) fn read(intent: &str) -> Reading {
    let mut r = Reading::default();
    let mut before = String::new();
    for written in split(intent) {
        let word = fold(written);
        if is(&word, ADD) {
            r.parts.insert(Part::Entry);
            r.add.get_or_insert_with(|| written.to_lowercase());
        } else if is(&word, CHOICE) {
            r.parts.insert(Part::Choice);
        } else if is(&word, COUNT) {
            r.parts.insert(Part::Count);
        } else if is(&word, LIST) {
            r.parts.insert(Part::List);
        } else if is(&word, MEDIA) {
            r.parts.insert(Part::Media);
        } else if is(&word, COMPARE) {
            r.parts.insert(Part::Media);
            r.compare = true;
        } else if is(&word, COLUMNS) {
            let one = is(&before, ONE);
            r.form = Some(if one { Form::Compact } else { Form::Columns });
        } else if is(&word, COMPACT) {
            r.form = Some(Form::Compact);
        } else if is(&word, EVERYTHING) {
            r.everything = true;
        } else if !is_filler(&word) {
            r.words.push((written.to_owned(), word.clone()));
        }
        before = word;
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_word_folds_its_case_accents_and_plural() {
        assert_eq!(fold("Estatísticas"), "estatistica");
        assert_eq!(fold("só"), "so");
        assert_eq!(fold("columns"), "column");
        assert_eq!(fold("is"), "is");
        assert_eq!(fold("Botão"), "botao");
    }

    #[test]
    fn the_grammar_reads_both_languages() {
        let en = read("only what is active and the add field");
        let pt = read("só o que está ativo e o campo de adicionar");
        assert_eq!(en.parts, BTreeSet::from([Part::Entry]));
        assert_eq!(pt.parts, en.parts);
        assert_eq!(en.words, [("active".to_owned(), "active".to_owned())]);
        assert_eq!(pt.words, [("ativo".to_owned(), "ativo".to_owned())]);
        assert_eq!(en.add.as_deref(), Some("add"));
        assert_eq!(pt.add.as_deref(), Some("adicionar"));
        assert_eq!(read("duas colunas").form, Some(Form::Columns));
        assert_eq!(read("two columns").form, Some(Form::Columns));
        assert_eq!(read("uma coluna").form, Some(Form::Compact));
        assert_eq!(read("compacto").form, Some(Form::Compact));
        let counts = read("contagem e filtros");
        assert_eq!(counts.parts, BTreeSet::from([Part::Choice, Part::Count]));
        for list in ["the items", "os itens", "a list", "a lista", "item"] {
            assert_eq!(read(list).parts, BTreeSet::from([Part::List]), "{list}");
        }
        assert!(read("tudo").everything);
        assert!(read("  ,  ").is_empty());
        assert!(read("the only field").is_empty());
        for media in ["the video", "o vídeo", "a frame", "the images"] {
            assert_eq!(read(media).parts, BTreeSet::from([Part::Media]), "{media}");
            assert!(!read(media).compare);
        }
        let compare = read("compare the original with the cuts");
        assert!(compare.compare);
        assert_eq!(compare.parts, BTreeSet::from([Part::Media]));
        assert_eq!(
            compare.words,
            [
                ("original".to_owned(), "original".to_owned()),
                ("cuts".to_owned(), "cut".to_owned())
            ]
        );
        assert!(read("comparar").compare);
    }
}
