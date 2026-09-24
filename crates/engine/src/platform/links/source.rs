//! Who delivers a link to the app's loop (task-76 item 6), by the shape
//! of the other platform modules: one trait ([`LinkSource`], `poll`
//! without blocking), the launch argument ([`LaunchLinks`], every
//! desktop), a fake for the tests ([`FakeLinks`]), none on wasm and
//! mobile ([`NoLinks`]), [`default_links`] choosing.
//!
//! What each platform delivers: windows (a registry key with `URL
//! Protocol` and `"<exe>" "%1"`) and linux (a `.desktop` with
//! `x-scheme-handler/<scheme>` and `%u`) start a new process with the
//! link as its argument, the app open or not; a second instance is a
//! second window, and passing the link to the one that runs is the
//! platform bring-up's. macOS hands a registered scheme to a bundled app
//! as a `GURL` Apple event, to the instance that runs; that handler is
//! the bring-up's too (it needs the bundle, and objc bindings the engine
//! does not name yet), and until then the launch argument is the way in
//! there as well (`cargo run -- <link>`). Registering the scheme with
//! the system is never done here.

use std::collections::VecDeque;
use std::ffi::OsString;
use std::fmt;
use std::sync::{Arc, Mutex};

use super::{Link, LinkError, excerpt, is_link};
use crate::platform::menu::locked;

/// A text that came as a link and is not one of this app's: what came,
/// shortened to one line a person can read ([`excerpt`]), and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused {
    pub link: String,
    pub why: LinkError,
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.link, self.why)
    }
}

/// A link as it is delivered: read, or refused with why.
pub type Delivered = Result<Link, Refused>;

/// `raw`, delivered to an app of `scheme`, read.
pub fn read(raw: &str, scheme: &str) -> Delivered {
    Link::parse(raw, scheme).map_err(|why| Refused {
        link: excerpt(raw),
        why,
    })
}

/// Where links come from. `poll` hands back every link delivered since
/// the last poll, read, in the order they came, and never blocks; the
/// app polls on its loop, and a source that delivers while the app runs
/// wakes the loop first (the `wake` of [`default_links`]).
pub trait LinkSource {
    fn poll(&mut self) -> Vec<Delivered>;
}

/// Texts waiting to be read as links of one scheme.
#[derive(Debug)]
struct Waiting {
    scheme: String,
    texts: VecDeque<String>,
}

impl Waiting {
    fn new(scheme: &str) -> Waiting {
        Waiting {
            scheme: scheme.to_owned(),
            texts: VecDeque::new(),
        }
    }

    fn drain(&mut self) -> Vec<Delivered> {
        let scheme = &self.scheme;
        self.texts.drain(..).map(|raw| read(&raw, scheme)).collect()
    }
}

// -- the launch argument -----------------------------------------------------

/// The links an app was started with: the arguments that start as links
/// of its scheme do ([`is_link`], `<scheme>://`), in order, whatever
/// follows (a malformed one is delivered refused, so the person hears of
/// it); the other arguments are the app's. Each is delivered once, on
/// the first poll.
#[derive(Debug)]
pub struct LaunchLinks {
    waiting: Waiting,
}

impl LaunchLinks {
    /// The links among `args` (the program's name left out by the
    /// caller). An argument that is not UTF-8 is read with its bad bytes
    /// replaced, so a link among them is refused, never skipped.
    pub fn from_args<I, S>(scheme: &str, args: I) -> LaunchLinks
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let mut waiting = Waiting::new(scheme);
        for arg in args {
            let arg = arg.into().to_string_lossy().into_owned();
            if is_link(&arg, scheme) {
                waiting.texts.push_back(arg);
            }
        }
        LaunchLinks { waiting }
    }

    /// The links among this process's arguments.
    pub fn from_env(scheme: &str) -> LaunchLinks {
        LaunchLinks::from_args(scheme, std::env::args_os().skip(1))
    }

    /// Links not polled yet.
    pub fn pending(&self) -> usize {
        self.waiting.texts.len()
    }
}

impl LinkSource for LaunchLinks {
    fn poll(&mut self) -> Vec<Delivered> {
        self.waiting.drain()
    }
}

// -- the fake ------------------------------------------------------------------

/// The links of a test: [`FakeLinks::open`] is someone opening a text as
/// a link of the app (a web page, a mail, another app, a person at the
/// terminal), and `poll` hands it back read, as a platform would.
#[derive(Debug)]
pub struct FakeLinks {
    waiting: Waiting,
}

impl FakeLinks {
    pub fn new(scheme: &str) -> FakeLinks {
        FakeLinks {
            waiting: Waiting::new(scheme),
        }
    }

    /// `raw` is opened as a link of the app; queued for `poll`.
    pub fn open(&mut self, raw: impl Into<String>) -> &mut FakeLinks {
        self.waiting.texts.push_back(raw.into());
        self
    }

    /// Links opened and not polled yet.
    pub fn pending(&self) -> usize {
        self.waiting.texts.len()
    }
}

impl LinkSource for FakeLinks {
    fn poll(&mut self) -> Vec<Delivered> {
        self.waiting.drain()
    }
}

// -- no link at all --------------------------------------------------------------

/// The links of a target without a way in yet: wasm (the page's own URL
/// is another matter) and mobile (the iOS and android shells deliver
/// links through their own entry, the bring-up's). Nothing ever comes.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoLinks;

impl LinkSource for NoLinks {
    fn poll(&mut self) -> Vec<Delivered> {
        Vec::new()
    }
}

/// A boxed source is one too.
impl<L: LinkSource + ?Sized> LinkSource for Box<L> {
    fn poll(&mut self) -> Vec<Delivered> {
        (**self).poll()
    }
}

/// A source shared by handle: the app polls one `Arc`, a test keeps the
/// other and opens links. A poisoned lock still answers: a queue of texts
/// holds no invariant a panic elsewhere could break.
impl<L: LinkSource> LinkSource for Arc<Mutex<L>> {
    fn poll(&mut self) -> Vec<Delivered> {
        locked(self).poll()
    }
}

/// The links of the current target for an app of `scheme`: the launch
/// argument on the desktop, read now (call it once the app can show a
/// refusal: its main window is up), none on wasm and mobile. `wake` is
/// for a source that delivers while the app runs (a shell sends itself
/// an event through its loop proxy and polls on the loop); the launch
/// argument, read once, never calls it.
pub fn default_links(scheme: &str, wake: impl Fn() + Send + Sync + 'static) -> Box<dyn LinkSource> {
    let _ = wake;
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    {
        Box::new(LaunchLinks::from_env(scheme))
    }
    #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
    {
        let _ = scheme;
        Box::new(NoLinks)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::platform::links::MAX_LEN;

    const SCHEME: &str = "plev-test";

    fn ok(delivered: &Delivered) -> &Link {
        delivered.as_ref().expect("a link")
    }

    #[test]
    fn the_launch_arguments_that_start_as_links_are_links_in_order_and_once() {
        let args = [
            "Todo",
            "plev-test://shell.page/select?value=Files",
            "dark",
            "PLEV-TEST://shell.theme/select?value=dark",
            "plev-test://shell.page/select#top",
            "https://example.com/plev-test://x/y",
            "-psn_0_12345",
        ];
        let mut launch = LaunchLinks::from_args(SCHEME, args);
        assert_eq!(launch.pending(), 3, "the others are the app's arguments");
        let delivered = launch.poll();
        assert_eq!(delivered.len(), 3);
        assert_eq!(ok(&delivered[0]).target.as_str(), "shell.page");
        assert_eq!(ok(&delivered[0]).value.as_deref(), Some("Files"));
        assert_eq!(ok(&delivered[1]).target.as_str(), "shell.theme");
        let refused = delivered[2].as_ref().expect_err("a fragment");
        assert_eq!(refused.why, LinkError::Fragment);
        assert_eq!(refused.link, "plev-test://shell.page/select#top");
        assert!(
            refused
                .to_string()
                .starts_with("plev-test://shell.page/select#top: "),
            "{refused}"
        );
        assert!(launch.poll().is_empty(), "delivered once");
        assert_eq!(launch.pending(), 0);
    }

    #[test]
    fn an_argument_that_is_not_utf8_is_refused_never_skipped() {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            let bad = OsString::from_vec(b"plev-test://shell.page/select?value=\xff".to_vec());
            let mut launch = LaunchLinks::from_args(SCHEME, [bad]);
            let delivered = launch.poll();
            assert_eq!(delivered.len(), 1);
            let refused = delivered[0].as_ref().expect_err("a bad byte");
            assert!(
                matches!(
                    refused.why,
                    LinkError::BadChar {
                        found: '\u{fffd}',
                        ..
                    }
                ),
                "{refused:?}"
            );
        }
    }

    #[test]
    fn a_refused_link_is_one_short_line_whatever_came() {
        let long = format!(
            "plev-test://shell.page/select?value={}",
            "a".repeat(MAX_LEN)
        );
        let mut fake = FakeLinks::new(SCHEME);
        fake.open(long.as_str())
            .open("plev-test://shell.page/select?value=x\ny")
            .open("plev-test://shell.page/sel\u{202e}ect");
        let delivered = fake.poll();
        let lines: Vec<String> = delivered
            .iter()
            .map(|d| d.as_ref().expect_err("refused").to_string())
            .collect();
        assert!(matches!(
            delivered[0].as_ref().expect_err("long").why,
            LinkError::TooLong { .. }
        ));
        for line in &lines {
            assert!(line.len() < 300, "{line}");
            assert!(!line.contains('\n') && !line.contains('\u{202e}'), "{line}");
        }
        assert!(lines[1].contains("\\n"), "{}", lines[1]);
        assert!(lines[2].contains("\\u{202e}"), "{}", lines[2]);
    }

    #[test]
    fn the_fake_delivers_what_was_opened_in_order_and_is_shared_by_handle() {
        let shared = Arc::new(Mutex::new(FakeLinks::new(SCHEME)));
        let mut app: Box<dyn LinkSource> = Box::new(shared.clone());
        assert!(app.poll().is_empty());
        locked(&shared)
            .open("plev-test://shell.theme/select?value=dark")
            .open("plev-test://nope/x/y");
        assert_eq!(locked(&shared).pending(), 2);
        let delivered = app.poll();
        assert_eq!(ok(&delivered[0]).action, "select");
        assert_eq!(
            delivered[1].as_ref().expect_err("a slash more").why,
            LinkError::ExtraPath
        );
        assert_eq!(locked(&shared).pending(), 0);
        assert!(app.poll().is_empty());
    }

    #[test]
    fn no_links_is_nothing_and_the_default_reads_this_process() {
        assert!(NoLinks.poll().is_empty());
        // the test harness's own arguments are no links of this scheme
        let mut default = default_links(SCHEME, || {});
        assert!(default.poll().is_empty());
    }
}
