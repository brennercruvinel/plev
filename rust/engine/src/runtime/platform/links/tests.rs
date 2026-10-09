//! The grammar of a link, case by case: what reads, what is refused and
//! why, and the form a link is shown in.

use sem::{Actor, Policy, Role, SnapshotBuilder, Value};

use super::*;

const SCHEME: &str = "plev-test";

fn parse(url: &str) -> Result<Link, LinkError> {
    Link::parse(url, SCHEME)
}

fn link(target: &str, action: &str, value: Option<&str>) -> Link {
    Link {
        target: SemId::parse(target).expect("a valid id"),
        action: action.to_owned(),
        value: value.map(str::to_owned),
    }
}

#[test]
fn a_link_names_a_capability_an_action_and_maybe_a_value() {
    assert_eq!(
        parse("plev-test://shell.page/select?value=Files"),
        Ok(link("shell.page", "select", Some("Files")))
    );
    assert_eq!(
        parse("plev-test://shell.detach/toggle"),
        Ok(link("shell.detach", "toggle", None))
    );
    // a key segment, raw or encoded; a key may hold any text
    assert_eq!(
        parse("plev-test://todo.item[3]/toggle"),
        Ok(link("todo.item[3]", "toggle", None))
    );
    assert_eq!(
        parse("plev-test://changes.file%5Bsrc%2Flib.rs%5D/stage"),
        Ok(link("changes.file[src/lib.rs]", "stage", None))
    );
    // text is percent-encoded UTF-8; a plus is a plus; empty is a value
    assert_eq!(
        parse("plev-test://shell.page/select?value=Data%20Table"),
        Ok(link("shell.page", "select", Some("Data Table")))
    );
    assert_eq!(
        parse("plev-test://todo.input/set_value?value=caf%C3%A9+1"),
        Ok(link("todo.input", "set_value", Some("café+1")))
    );
    assert_eq!(
        parse("plev-test://todo.input/set_value?value="),
        Ok(link("todo.input", "set_value", Some("")))
    );
    // the scheme in any ASCII case (RFC 3986, 3.1)
    assert_eq!(
        parse("PLEV-Test://shell.theme/select?value=dark"),
        Ok(link("shell.theme", "select", Some("dark")))
    );
}

#[test]
fn what_is_not_a_link_is_refused_with_why() {
    let cases: &[(&str, LinkError)] = &[
        ("shell.page/select", LinkError::NotALink),
        ("plev-test:shell.page/select", LinkError::NotALink),
        (
            "https://shell.page/select",
            LinkError::OtherScheme {
                found: "https".to_owned(),
            },
        ),
        (
            "plev://shell.page/select",
            LinkError::OtherScheme {
                found: "plev".to_owned(),
            },
        ),
        ("plev-test://shell.page/select#top", LinkError::Fragment),
        (
            "plev-test://shell.page/select?value=a#b",
            LinkError::Fragment,
        ),
        ("plev-test://me@shell.page/select", LinkError::UserInfo),
        (
            "plev-test://me:secret@shell.page/select",
            LinkError::UserInfo,
        ),
        ("plev-test://shell.page:80/select", LinkError::Port),
        ("plev-test://shell.page/select/", LinkError::ExtraPath),
        ("plev-test://shell.page/select/more", LinkError::ExtraPath),
        ("plev-test://shell.page//select", LinkError::ExtraPath),
        ("plev-test:///select", LinkError::NoTarget),
        ("plev-test://", LinkError::NoTarget),
        ("plev-test://shell.page", LinkError::NoAction),
        ("plev-test://shell.page/", LinkError::NoAction),
        ("plev-test://shell.page/?value=x", LinkError::NoAction),
        (
            "plev-test://shell.page/Select",
            LinkError::BadAction {
                found: "Select".to_owned(),
            },
        ),
        (
            "plev-test://shell.page/sel-ect",
            LinkError::BadAction {
                found: "sel-ect".to_owned(),
            },
        ),
        (
            "plev-test://shell.page/select2",
            LinkError::BadAction {
                found: "select2".to_owned(),
            },
        ),
        (
            "plev-test://shell.page/s%65lect",
            LinkError::BadAction {
                found: "s%65lect".to_owned(),
            },
        ),
        ("plev-test://shell.page/select?", LinkError::NoValue),
        ("plev-test://shell.page/select?value", LinkError::NoValue),
        (
            "plev-test://shell.page/select?page=Files",
            LinkError::UnknownKey {
                key: "page".to_owned(),
            },
        ),
        (
            "plev-test://shell.page/select?Value=Files",
            LinkError::UnknownKey {
                key: "Value".to_owned(),
            },
        ),
        (
            "plev-test://shell.page/select?flag",
            LinkError::UnknownKey {
                key: "flag".to_owned(),
            },
        ),
        (
            "plev-test://shell.page/select?value=a&value=b",
            LinkError::ExtraKey,
        ),
        (
            "plev-test://shell.page/select?value=a&then=b",
            LinkError::ExtraKey,
        ),
    ];
    for (url, why) in cases {
        assert_eq!(parse(url).as_ref(), Err(why), "{url}");
    }
}

#[test]
fn an_id_that_is_not_an_id_is_refused() {
    for url in [
        "plev-test://Shell.page/select",
        "plev-test://shell..page/select",
        "plev-test://shell.page[x/select",
        "plev-test://%5Bx%5D/select",
        "plev-test://shell%20page/select",
        "plev-test://shell.page./select",
    ] {
        assert!(
            matches!(parse(url), Err(LinkError::BadTarget { .. })),
            "{url}: {:?}",
            parse(url)
        );
    }
    let why = parse("plev-test://Shell.page/select").expect_err("uppercase");
    assert_eq!(
        why.to_string(),
        "the link's id is not an id: invalid character 'S' at byte 0"
    );
}

#[test]
fn raw_text_is_printable_ascii_and_the_rest_travels_encoded() {
    assert_eq!(
        parse("plev-test://shell.page/select?value=Data Table"),
        Err(LinkError::BadChar { at: 40, found: ' ' })
    );
    assert_eq!(
        parse("plev-test://shell.page/select?value=café"),
        Err(LinkError::BadChar {
            at: 39, found: 'é'
        })
    );
    assert!(matches!(
        parse("plev-test://shell.page/select?value=a\nb"),
        Err(LinkError::BadChar { found: '\n', .. })
    ));
    assert!(matches!(
        parse("\tplev-test://shell.page/select"),
        Err(LinkError::BadChar { at: 0, .. })
    ));
}

#[test]
fn a_percent_escape_is_two_hex_digits_of_utf8_that_shows_in_one_line() {
    let bad = |escape: &str| LinkError::Percent {
        escape: escape.to_owned(),
    };
    assert_eq!(
        parse("plev-test://shell.page/select?value=%G0"),
        Err(bad("%G0"))
    );
    assert_eq!(
        parse("plev-test://shell.page/select?value=%4"),
        Err(bad("%4"))
    );
    assert_eq!(
        parse("plev-test://shell.page/select?value=ab%"),
        Err(bad("%"))
    );
    assert_eq!(
        parse("plev-test://shell.page/select?value=%+1"),
        Err(bad("%+1"))
    );
    // the id is cut at its slash before it is decoded
    assert_eq!(parse("plev-test://shell%2/select"), Err(bad("%2")));
    // bytes that are not UTF-8
    assert_eq!(
        parse("plev-test://shell.page/select?value=%FF"),
        Err(LinkError::Utf8)
    );
    assert_eq!(
        parse("plev-test://shell.page/select?value=%C3"),
        Err(LinkError::Utf8)
    );
    // a newline, a NUL, DEL, a direction override: a second line or a
    // line read in another order
    for escape in [
        "%0A",
        "%0D",
        "%00",
        "%7F",
        "%C2%85",
        "%E2%80%AE",
        "%E2%81%A6",
    ] {
        let url = format!("plev-test://shell.page/select?value=a{escape}b");
        assert_eq!(parse(&url), Err(LinkError::Control), "{url}");
    }
    assert_eq!(
        parse("plev-test://todo.item%5B%0A%5D/toggle"),
        Err(LinkError::Control)
    );
}

#[test]
fn a_link_has_at_most_2048_bytes() {
    let head = "plev-test://shell.page/select?value=";
    let at_most = format!("{head}{}", "a".repeat(MAX_LEN - head.len()));
    assert_eq!(at_most.len(), MAX_LEN);
    assert!(parse(&at_most).is_ok());
    let over = format!("{at_most}a");
    assert_eq!(parse(&over), Err(LinkError::TooLong { len: MAX_LEN + 1 }));
    // the length is looked at first, before any character
    let huge = format!("{head}{}", "\u{202e}".repeat(MAX_LEN));
    assert!(matches!(parse(&huge), Err(LinkError::TooLong { .. })));
}

#[test]
fn a_link_is_shown_in_the_form_it_reads_back() {
    for url in [
        "plev-test://shell.page/select?value=Data%20Table",
        "plev-test://todo.item[3]/toggle",
        "plev-test://changes.file[src%2Flib.rs]/stage",
        "plev-test://todo.input/set_value?value=caf%C3%A9%2B1%26%3D%3F%23",
        "plev-test://todo.input/set_value?value=",
    ] {
        let read = parse(url).expect("a link");
        assert_eq!(read.url(SCHEME), url);
        assert_eq!(parse(&read.url(SCHEME)), Ok(read));
    }
    // the canonical form of an encoded bracket and a raw plus
    let read = parse("plev-test://todo.item%5B3%5D/toggle?value=a+b").expect("a link");
    assert_eq!(
        read.url(SCHEME),
        "plev-test://todo.item[3]/toggle?value=a%2Bb"
    );
}

#[test]
fn the_start_of_a_link_tells_it_from_another_argument() {
    assert!(is_link("plev-test://shell.page/select", SCHEME));
    assert!(is_link("PLEV-TEST://x", SCHEME));
    assert!(is_link("plev-test://", SCHEME), "then refused by the parse");
    assert!(!is_link("plev-test:/x", SCHEME));
    assert!(!is_link("plev-test", SCHEME));
    assert!(!is_link("plev-tests://x", SCHEME));
    assert!(!is_link("Todo", SCHEME));
    assert!(!is_link("é-test://x", SCHEME), "no char boundary panic");
    assert!(!is_link("", SCHEME));
}

#[test]
fn an_excerpt_is_one_short_printable_line() {
    assert_eq!(excerpt("plev-test://a/b"), "plev-test://a/b");
    assert_eq!(excerpt("a\nb\u{202e}c"), "a\\nb\\u{202e}c");
    let long = excerpt(&"x".repeat(500));
    assert_eq!(long.len(), 99);
    assert!(long.ends_with("..."));
}

#[test]
fn a_links_request_is_an_agents_named_link_never_a_persons() {
    let mut b = SnapshotBuilder::new();
    b.node("shell", Role::Region, "shell");
    b.node("shell.page", Role::Select, "page")
        .parent("shell")
        .state("selected", "Todo")
        .action("select")
        .reads("shell.page:selected");
    b.node("shell.print", Role::Button, "print")
        .parent("shell")
        .action("activate")
        .policy(Policy::default());
    let seen = b.finish().expect("valid");
    let read = parse("plev-test://shell.page/select?value=Files").expect("a link");
    let req = read.request(&seen).expect("observed");
    assert_eq!(
        req.actor,
        Actor::Agent {
            name: "link".to_owned()
        }
    );
    assert_ne!(req.actor, Actor::Human);
    assert_eq!(req.actor, actor());
    assert_eq!(req.target.as_str(), "shell.page");
    assert_eq!(req.action.as_str(), "select");
    assert_eq!(req.param, Some(Value::Text("Files".to_owned())));
    assert_eq!(req.observed.values.len(), 1, "what it read, as it saw it");
    assert!(req.ask.is_none(), "a link never asks a person on its own");
    // a capability the projection does not have makes no request
    let gone = parse("plev-test://shell.detach/toggle").expect("a link");
    assert!(gone.request(&seen).is_err());
}

#[test]
fn the_open_list_is_the_apps_by_id_and_action() {
    let open = [("shell.page", "select"), ("shell.theme", "select")];
    assert!(link("shell.page", "select", Some("Files")).is_in(&open));
    assert!(link("shell.theme", "select", None).is_in(&open));
    assert!(!link("shell.page", "toggle", None).is_in(&open));
    assert!(!link("shell.print", "activate", None).is_in(&open));
    assert!(!link("shell.page.x", "select", None).is_in(&open));
    assert!(!link("shell.page", "select", None).is_in(&[]));
}

#[test]
fn every_error_says_why_in_words() {
    let all = [
        LinkError::TooLong { len: 3000 },
        LinkError::BadChar { at: 3, found: ' ' },
        LinkError::NotALink,
        LinkError::OtherScheme {
            found: "https".to_owned(),
        },
        LinkError::Fragment,
        LinkError::UserInfo,
        LinkError::Port,
        LinkError::ExtraPath,
        LinkError::NoTarget,
        LinkError::BadTarget {
            reason: "empty id".to_owned(),
        },
        LinkError::NoAction,
        LinkError::BadAction {
            found: "X".to_owned(),
        },
        LinkError::NoValue,
        LinkError::UnknownKey {
            key: "page".to_owned(),
        },
        LinkError::ExtraKey,
        LinkError::Percent {
            escape: "%G".to_owned(),
        },
        LinkError::Utf8,
        LinkError::Control,
    ];
    let texts: Vec<String> = all.iter().map(ToString::to_string).collect();
    for (n, text) in texts.iter().enumerate() {
        assert!(!text.is_empty() && !text.contains('\n'), "{text}");
        assert!(
            texts.iter().skip(n + 1).all(|other| other != text),
            "{text} said twice"
        );
    }
    assert_eq!(
        texts[0],
        "the link has 3000 bytes, above the 2048 a link may have"
    );
}
