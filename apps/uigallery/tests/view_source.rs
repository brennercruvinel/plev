//! View source for pixels on the real gallery (task-65, phases a and b): a
//! point of a staged frame leads to the code, with no window and no editor.
//! The literals are the workspace's own sources, read once, as the window's
//! watcher reads them. Only a desktop debug build has any of this (the
//! engine's `hot_reload`), so the file is empty everywhere else.
#![cfg(all(
    debug_assertions,
    any(target_os = "macos", target_os = "linux", target_os = "windows")
))]

use std::sync::Once;

use accord::SemId;
use engine::drawing::compositor::{Compositor, SceneNode};
use engine::drawing::headless::Viewport;
use engine::runtime::hot_reload::view_source::{Origin, origin_at, text_at};
use engine::runtime::hot_reload::{Site, literals, source_dirs};
use uigallery::shell::{StagedPage, stage_page};

/// The Todo page's subtitle, a string literal written once in the sources.
const SUBTITLE: &str = "A complete small todo app: domain state, per-item animation, input.";

fn staged(page: &str) -> StagedPage {
    static BASELINE: Once = Once::new();
    BASELINE.call_once(|| literals::take_baseline(&source_dirs()));
    stage_page(Viewport::new(1200.0, 800.0, 1.0), page, "core", None).unwrap()
}

/// The middle of where `id` shows in the frame.
fn center(c: &Compositor, id: &str) -> (f32, f32) {
    let id = SemId::parse(id).unwrap();
    let [x, y, w, h] = c.binding(&id).unwrap().visible_rect().unwrap();
    (x + w / 2.0, y + h / 2.0)
}

/// A point inside the first text run drawn with `text`.
fn on_text(c: &Compositor, text: &str) -> (f32, f32) {
    c.layers()
        .iter()
        .flat_map(|l| l.nodes())
        .find_map(|n| match n {
            SceneNode::Text { key, x, y, .. } if key.text == text => Some((x + 3.0, y + 5.0)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no text {text:?} in the frame"))
}

/// The source at `site`, from its column to the end of its line.
fn source_at(site: &Site) -> String {
    let src = std::fs::read_to_string(&site.file).unwrap();
    let line = src.lines().nth(site.line as usize - 1).unwrap();
    line.chars().skip(site.column as usize - 1).collect()
}

fn origin_of(page: &StagedPage, (x, y): (f32, f32)) -> Origin {
    origin_at(page.compositor(), page.semantics().ok().as_ref(), x, y)
}

#[test]
fn the_field_that_adds_leads_to_where_the_todo_projects_it() {
    let page = staged("Todo");
    let origin = origin_of(&page, center(page.compositor(), "todo.input"));
    let Origin::Projected { id, site } = &origin else {
        panic!("{origin}");
    };
    assert_eq!(id.as_str(), "todo.input");
    assert!(
        site.file
            .ends_with("apps/uigallery/src/views/apps/todo/semantics/mod.rs")
    );
    assert!(
        source_at(site).starts_with("project(b, &ids.input, \"new todo\""),
        "{origin}: {}",
        source_at(site)
    );
    assert_eq!(origin.site(), Some(site));
}

#[test]
fn an_item_leads_to_the_list_item_projection() {
    let page = staged("Todo");
    let origin = origin_of(&page, center(page.compositor(), "todo.item[1]"));
    let Origin::Projected { id, site } = &origin else {
        panic!("{origin}");
    };
    assert_eq!(id.as_str(), "todo.item[1]");
    assert!(
        site.file
            .ends_with("apps/uigallery/src/views/apps/todo/semantics/mod.rs")
    );
    assert!(
        source_at(site).starts_with("node(&id, Role::ListItem, item.text())"),
        "{origin}: {}",
        source_at(site)
    );
    // the item's own text is inside its row: the control wins
    let label = on_text(page.compositor(), "Absorb the plev demos into the showcase");
    assert_eq!(origin_of(&page, label), origin);
}

#[test]
fn a_static_title_leads_to_its_one_literal() {
    let page = staged("Todo");
    for shown in ["Todo", SUBTITLE] {
        let origin = origin_of(&page, on_text(page.compositor(), shown));
        let Origin::Literal { text, sites } = &origin else {
            panic!("{origin}");
        };
        assert_eq!(text, shown);
        assert_eq!(sites.len(), 1, "{origin}");
        let site = &sites[0];
        assert!(
            site.file
                .ends_with("apps/uigallery/src/views/pages/table.rs")
        );
        assert!(
            source_at(site).starts_with(&format!("{shown:?}")),
            "{origin}: {}",
            source_at(site)
        );
        assert_eq!(origin.site(), Some(site));
    }
}

#[test]
fn a_text_several_literals_hold_is_listed_and_not_opened() {
    let page = staged("Todo");
    let origin = origin_of(&page, on_text(page.compositor(), "plev"));
    let Origin::Literal { text, sites } = &origin else {
        panic!("{origin}");
    };
    assert_eq!(text, "plev");
    assert!(sites.len() > 1, "{origin}");
    assert!(
        sites.iter().all(|s| source_at(s).starts_with("\"plev\"")),
        "{origin}"
    );
    assert!(
        sites.is_sorted_by(|a, b| (&a.file, a.line) <= (&b.file, b.line)),
        "{origin}"
    );
    assert_eq!(origin.site(), None);
    assert!(origin.to_string().contains("literals hold it, none opened"));
}

#[test]
fn a_formatted_text_has_no_literal() {
    let page = staged("Slider");
    let shown = "step 1: 4";
    let origin = origin_of(&page, on_text(page.compositor(), shown));
    assert_eq!(origin, Origin::Formatted(shown.to_owned()));
    assert_eq!(origin.site(), None);

    // a row of the data table is formatted too, and since the component
    // pages project (task-75 phase 2b) it is a bound row: it leads to the
    // row the page projects
    let page = staged("Data Table");
    let origin = origin_of(
        &page,
        on_text(page.compositor(), "crates/comps/src/file_0.rs"),
    );
    let Origin::Projected { id, site } = &origin else {
        panic!("{origin}");
    };
    assert_eq!(id.as_str(), "table_page.table[0]");
    assert!(
        site.file
            .ends_with("apps/uigallery/src/views/components/table.rs")
    );

    // the todo's counter is formatted too, but it sits inside the bound
    // card: the control is what a click there finds
    let page = staged("Todo");
    let (x, y) = on_text(page.compositor(), "2 active · 1 done");
    assert_eq!(
        text_at(page.compositor(), x, y).as_deref(),
        Some("2 active · 1 done")
    );
    let Origin::Projected { id, .. } = origin_of(&page, (x, y)) else {
        panic!("the card is bound");
    };
    assert_eq!(id.as_str(), "todo");
}
