//! The page table's invariants.

use super::*;

#[test]
fn pages_are_grouped_in_order_and_components_are_alphabetical() {
    let mut last = 0;
    for p in PAGES {
        let g = Group::ALL.iter().position(|g| *g == p.group).unwrap();
        assert!(g >= last, "{} is out of group order", p.title);
        last = g;
    }
    let comps: Vec<&str> = PAGES
        .iter()
        .filter(|p| p.group == Group::Components)
        .map(|p| p.title)
        .collect();
    let mut sorted = comps.clone();
    sorted.sort_unstable();
    assert_eq!(comps, sorted);
}

#[test]
fn every_pane_has_exactly_one_page_of_its_own_and_titles_are_unique() {
    let own: Vec<Pane> = PAGES
        .iter()
        .filter_map(|p| match p.body {
            Body::Pane(pane) => Some(pane),
            _ => None,
        })
        .collect();
    for (i, pane) in own.iter().enumerate() {
        assert!(!own[i + 1..].contains(pane), "{pane:?} owns two pages");
    }
    let mut titles: Vec<&str> = PAGES.iter().map(|p| p.title).collect();
    titles.sort_unstable();
    titles.dedup();
    assert_eq!(titles.len(), PAGES.len());
}

#[test]
fn find_is_case_insensitive() {
    assert_eq!(find("theme"), Some(0));
    let button = PAGES.iter().position(|p| p.pane() == Some(Pane::Button));
    assert_eq!(find("BUTTON"), button);
    assert_eq!(find("nope"), None);
}
