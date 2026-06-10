use narrate::plev_narrate;

#[test]
fn simple_div() {
    let _elem = plev_narrate! {
        div bg "blue"
    };
}

#[test]
fn text_with_show() {
    let _elem = plev_narrate! {
        text { show "Hello, World!" }
    };
}

#[test]
fn nested_elements() {
    let _elem = plev_narrate! {
        div bg "slate-900" {
            text font_size 24, text_color "white" {
                show "Title"
            }
        }
    };
}

#[test]
fn row_and_col() {
    let _elem = plev_narrate! {
        col centered, gap 4, p 8 {
            row gap 2 {
                button px 4, py 2, bg "blue-500", rounded "md" {
                    show "OK"
                }
                button px 4, py 2, bg "red-500", rounded "md" {
                    show "Cancel"
                }
            }
        }
    };
}

#[test]
fn format_interpolation() {
    let count = 42;
    let _elem = plev_narrate! {
        text {
            show "Count: {count}"
        }
    };
}

#[test]
fn on_click_handler() {
    use std::cell::Cell;
    use std::rc::Rc;
    let clicked = Rc::new(Cell::new(false));
    let clicked2 = clicked.clone();
    let _elem = plev_narrate! {
        button bg "blue" {
            on click |_e| { clicked2.set(true) }
            show "Click me"
        }
    };
    assert!(!clicked.get());
}

#[test]
fn when_conditional() {
    let is_empty = true;
    let _elem = plev_narrate! {
        div {
            when { is_empty } {
                text { show "Nothing here" }
            }
        }
    };
}

#[test]
fn when_otherwise() {
    let is_empty = false;
    let _elem = plev_narrate! {
        div {
            when { is_empty } {
                text { show "Empty" }
            } otherwise {
                text { show "Has content" }
            }
        }
    };
}

#[test]
fn each_iteration() {
    let items = vec!["a", "b", "c"];
    let _elem = plev_narrate! {
        col {
            each _item in { items.clone() } {
                text { show "item" }
            }
        }
    };
}

#[test]
fn flag_modifiers() {
    let _elem = plev_narrate! {
        text bold, italic {
            show "Styled"
        }
    };
}

#[test]
fn expr_modifier_value() {
    let spacing = 8;
    let _elem = plev_narrate! {
        div gap { spacing } {
            text { show "spaced" }
        }
    };
}

#[test]
fn full_counter_example() {
    let count = 0;
    let _elem = plev_narrate! {
        col centered, gap 4, p 8, bg "slate-900" {
            text font_size 24, text_color "white" {
                show "Counter Demo"
            }
            row centered, gap 4 {
                button px 6, py 3, bg "blue-500", rounded "xl" {
                    on click { let _ = count; }
                    show "Increment"
                }
            }
            text font_size 48, bold, text_color "white" {
                show "Count: {count}"
            }
        }
    };
}
