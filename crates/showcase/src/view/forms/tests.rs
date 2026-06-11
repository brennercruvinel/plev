use super::*;

/// Every rect a layout hands out, flattened (for bounds checks).
fn all_rects(l: &Layout) -> Vec<Rect> {
    let mut v = vec![l.tabs, l.select];
    v.extend(l.checkboxes);
    v.extend(l.switches);
    v.extend(l.sliders);
    v.extend(l.progresses);
    v
}

/// Narrow viewport (~600px window): the columns must stack instead of
/// cropping column B, and nothing may reach past `content.w`.
#[test]
fn forms_columns_stack_in_narrow_content_without_overflow() {
    let theme = Theme::hoff();
    let section = FormsSection::new(&theme);
    let content = Rect::new(288.0, 80.0, 600.0, 700.0);
    let layout = section.layout(content);

    // Column B starts at the left edge, below column A.
    assert_eq!(layout.sliders[0].x, content.x, "column B must stack left");
    assert!(
        layout.sliders[0].y > layout.switches[2].y + layout.switches[2].h,
        "stacked column B must start below column A"
    );

    let right = content.x + content.w;
    for r in all_rects(&layout) {
        assert!(
            r.x + r.w <= right + 0.5,
            "rect {:?} overflows content right edge {right}",
            (r.x, r.y, r.w, r.h)
        );
    }
}

/// Wide viewport (~1600px window): the two columns must actually use
/// the available width instead of huddling at fixed offsets.
#[test]
fn forms_columns_spread_in_wide_content() {
    let theme = Theme::hoff();
    let section = FormsSection::new(&theme);
    let content = Rect::new(288.0, 80.0, 1272.0, 700.0);
    let layout = section.layout(content);

    // Two real columns, side by side.
    assert!(
        layout.sliders[0].x > content.x,
        "wide content must keep column B beside column A"
    );
    assert_eq!(layout.sliders[0].y - LABEL_H, content.y);

    // The columns span well past the old fixed 700px footprint.
    let span = layout.progresses[0].x + layout.progresses[0].w - content.x;
    assert!(
        span >= content.w * 0.6,
        "columns use only {span:.0}px of {:.0}px content",
        content.w
    );
    // …but each column stays clamped for legibility.
    assert!(layout.progresses[0].w <= COL_MAX_W);
    // The select follows its column instead of the old fixed x+380.
    assert_eq!(layout.select.x, layout.sliders[0].x);
    assert!(layout.select.w <= layout.progresses[0].w);
}

/// Every tab label must sit FOLGADO inside its segment: the GOLDEN_SPEC
/// flags cramped tabs as the broken state. We require at least 16px of
/// horizontal breathing room on each side (text centered in the segment).
#[test]
fn forms_tabs_keep_every_label_folgado() {
    let theme = Theme::hoff();
    let section = FormsSection::new(&theme);
    // A representative content rect (matches the page layout origin).
    let content = Rect::new(288.0, 80.0, 760.0, 700.0);
    let layout = section.layout(content);
    let rects = section.tabs.item_rects(layout.tabs);
    let style = TypographyScale::hoff().base_2sm();

    assert_eq!(rects.len(), section.tabs.labels.len());
    for (label, rect) in section.tabs.labels.iter().zip(&rects) {
        let (text_w, _) = TextMeasurer::measure_styled(label, &style, None);
        let slack = rect.w - text_w;
        assert!(
            slack >= 32.0,
            "tab '{label}' is cramped: segment {:.1}px holds {:.1}px of text ({:.1}px slack, need >=32)",
            rect.w,
            text_w,
            slack
        );
    }
}
