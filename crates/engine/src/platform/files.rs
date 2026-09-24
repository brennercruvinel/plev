//! File dialogs: pick a file, pick several, save, pick a folder, behind a
//! trait so an app asks the platform for a path and a test never opens a
//! window. [`NativeDialogs`] wraps `rfd` on the desktop (macOS, windows,
//! linux); [`FakeDialogs`] answers what a test loaded and keeps what was
//! asked; [`NoDialogs`] is the wasm and mobile answer, `Unsupported`.
//! The same shape as `clipboard.rs`: one trait, the real one under the
//! desktop cfg, the fake everywhere, [`default_dialogs`] choosing.
//!
//! Every call is synchronous and modal, as `rfd::FileDialog` is, and
//! answers a path: `Ok(None)` is the person cancelling. The browser's
//! picker gives bytes and only asynchronously (`rfd::AsyncFileDialog`),
//! which is another capability; on wasm this one is `Unsupported`.

use std::collections::VecDeque;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// A named set of extensions a dialog offers as one filter
/// ("urna corpus": `urna`, `nest`). Extensions without the dot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// What an app asks of a dialog: the title, the filters, the directory it
/// opens in and, for a save, the name it suggests. Every field is
/// optional; the platform's defaults fill the rest.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DialogRequest {
    pub title: Option<String>,
    pub filters: Vec<FileFilter>,
    pub directory: Option<PathBuf>,
    pub file_name: Option<String>,
}

impl DialogRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Adds a filter; the dialog shows every added one.
    pub fn filter(mut self, name: impl Into<String>, extensions: &[&str]) -> Self {
        self.filters.push(FileFilter {
            name: name.into(),
            extensions: extensions.iter().map(|e| (*e).to_owned()).collect(),
        });
        self
    }

    pub fn directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.directory = Some(directory.into());
        self
    }

    pub fn file_name(mut self, name: impl Into<String>) -> Self {
        self.file_name = Some(name.into());
        self
    }
}

/// Which dialog was asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogKind {
    PickFile,
    PickFiles,
    SaveFile,
    PickFolder,
}

impl fmt::Display for DialogKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            DialogKind::PickFile => "pick_file",
            DialogKind::PickFiles => "pick_files",
            DialogKind::SaveFile => "save_file",
            DialogKind::PickFolder => "pick_folder",
        })
    }
}

/// Why a dialog gave no answer at all (a cancel is `Ok(None)`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DialogError {
    /// This target has no native dialog (wasm, android, ios).
    Unsupported(DialogKind),
}

impl fmt::Display for DialogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(kind) => write!(f, "{kind}: no native file dialog on this target"),
        }
    }
}

impl std::error::Error for DialogError {}

/// What a dialog answers: the path or paths picked, `None` when the
/// person cancelled.
pub type Picked<T> = Result<Option<T>, DialogError>;

/// The four dialogs an app asks for. Each one blocks until the person
/// answers, on the thread that asked.
pub trait FileDialogs {
    fn pick_file(&mut self, req: &DialogRequest) -> Picked<PathBuf>;
    fn pick_files(&mut self, req: &DialogRequest) -> Picked<Vec<PathBuf>>;
    fn save_file(&mut self, req: &DialogRequest) -> Picked<PathBuf>;
    fn pick_folder(&mut self, req: &DialogRequest) -> Picked<PathBuf>;
}

// -- the fake -------------------------------------------------------------

/// What the fake answers a call with, in the order loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// The paths picked; a single-path dialog takes the first, and an
    /// empty list is a cancel.
    Paths(Vec<PathBuf>),
    Cancelled,
    Unsupported,
}

/// One call the fake took.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asked {
    pub kind: DialogKind,
    pub request: DialogRequest,
}

/// The dialogs of a test: answers what was loaded, in order, and keeps
/// every request. With nothing loaded every call is a cancel, so a page
/// under test that asks unexpectedly changes nothing.
#[derive(Debug, Default)]
pub struct FakeDialogs {
    answers: VecDeque<Answer>,
    asked: Vec<Asked>,
}

impl FakeDialogs {
    pub fn new() -> Self {
        Self::default()
    }

    /// The next call picks `path`.
    pub fn will_pick(mut self, path: impl Into<PathBuf>) -> Self {
        self.answers.push_back(Answer::Paths(vec![path.into()]));
        self
    }

    /// The next call picks all of `paths`.
    pub fn will_pick_all<P: Into<PathBuf>>(mut self, paths: impl IntoIterator<Item = P>) -> Self {
        self.answers
            .push_back(Answer::Paths(paths.into_iter().map(Into::into).collect()));
        self
    }

    /// The next call is cancelled.
    pub fn will_cancel(mut self) -> Self {
        self.answers.push_back(Answer::Cancelled);
        self
    }

    /// The next call answers `Unsupported`, as the wasm and mobile
    /// dialogs do.
    pub fn will_refuse(mut self) -> Self {
        self.answers.push_back(Answer::Unsupported);
        self
    }

    /// Every call taken so far, in order.
    pub fn asked(&self) -> &[Asked] {
        &self.asked
    }

    /// Every call taken so far, leaving none.
    pub fn take_asked(&mut self) -> Vec<Asked> {
        std::mem::take(&mut self.asked)
    }

    /// Answers left to give.
    pub fn pending(&self) -> usize {
        self.answers.len()
    }

    fn answer(&mut self, kind: DialogKind, req: &DialogRequest) -> Picked<Vec<PathBuf>> {
        self.asked.push(Asked {
            kind,
            request: req.clone(),
        });
        match self.answers.pop_front() {
            Some(Answer::Paths(paths)) if !paths.is_empty() => Ok(Some(paths)),
            Some(Answer::Unsupported) => Err(DialogError::Unsupported(kind)),
            Some(Answer::Paths(_)) | Some(Answer::Cancelled) | None => Ok(None),
        }
    }

    fn one(&mut self, kind: DialogKind, req: &DialogRequest) -> Picked<PathBuf> {
        Ok(self
            .answer(kind, req)?
            .and_then(|paths| paths.into_iter().next()))
    }
}

impl FileDialogs for FakeDialogs {
    fn pick_file(&mut self, req: &DialogRequest) -> Picked<PathBuf> {
        self.one(DialogKind::PickFile, req)
    }

    fn pick_files(&mut self, req: &DialogRequest) -> Picked<Vec<PathBuf>> {
        self.answer(DialogKind::PickFiles, req)
    }

    fn save_file(&mut self, req: &DialogRequest) -> Picked<PathBuf> {
        self.one(DialogKind::SaveFile, req)
    }

    fn pick_folder(&mut self, req: &DialogRequest) -> Picked<PathBuf> {
        self.one(DialogKind::PickFolder, req)
    }
}

// -- no dialog at all ------------------------------------------------------

/// The dialogs of a target without any: wasm (the browser's picker gives
/// bytes, asynchronously) and mobile (`rfd` has no backend there). Every
/// call is `Unsupported`, and the app says so where it would have opened
/// one.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoDialogs;

impl FileDialogs for NoDialogs {
    fn pick_file(&mut self, _req: &DialogRequest) -> Picked<PathBuf> {
        Err(DialogError::Unsupported(DialogKind::PickFile))
    }

    fn pick_files(&mut self, _req: &DialogRequest) -> Picked<Vec<PathBuf>> {
        Err(DialogError::Unsupported(DialogKind::PickFiles))
    }

    fn save_file(&mut self, _req: &DialogRequest) -> Picked<PathBuf> {
        Err(DialogError::Unsupported(DialogKind::SaveFile))
    }

    fn pick_folder(&mut self, _req: &DialogRequest) -> Picked<PathBuf> {
        Err(DialogError::Unsupported(DialogKind::PickFolder))
    }
}

/// Dialogs shared by handle: a test keeps one `Arc` and gives the page
/// the other as its `Box<dyn FileDialogs>`, then reads what the page
/// asked ([`FakeDialogs::asked`]). A poisoned lock still answers: the
/// dialogs hold no invariant a panic elsewhere could break.
impl<D: FileDialogs> FileDialogs for Arc<Mutex<D>> {
    fn pick_file(&mut self, req: &DialogRequest) -> Picked<PathBuf> {
        locked(self).pick_file(req)
    }

    fn pick_files(&mut self, req: &DialogRequest) -> Picked<Vec<PathBuf>> {
        locked(self).pick_files(req)
    }

    fn save_file(&mut self, req: &DialogRequest) -> Picked<PathBuf> {
        locked(self).save_file(req)
    }

    fn pick_folder(&mut self, req: &DialogRequest) -> Picked<PathBuf> {
        locked(self).pick_folder(req)
    }
}

fn locked<D>(shared: &Mutex<D>) -> MutexGuard<'_, D> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

// -- the native one ----------------------------------------------------------

/// The platform's dialogs through `rfd`: an `NSOpenPanel` or
/// `NSSavePanel` on macOS, the common dialogs on windows, the portal or
/// gtk on linux (the backend there is the linux bring-up's choice; the
/// engine turns `rfd`'s default features off). Modal on the calling
/// thread; no parent window is set, as the urnaui never set one.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeDialogs;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
impl NativeDialogs {
    pub fn new() -> Self {
        Self
    }

    /// The `rfd` builder for `req`. Building opens nothing; only the
    /// `pick_*` and `save_file` calls do.
    fn dialog(req: &DialogRequest) -> rfd::FileDialog {
        let mut dialog = rfd::FileDialog::new();
        if let Some(title) = &req.title {
            dialog = dialog.set_title(title);
        }
        for filter in &req.filters {
            dialog = dialog.add_filter(&filter.name, &filter.extensions);
        }
        if let Some(directory) = &req.directory {
            dialog = dialog.set_directory(directory);
        }
        if let Some(name) = &req.file_name {
            dialog = dialog.set_file_name(name);
        }
        dialog
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
impl FileDialogs for NativeDialogs {
    fn pick_file(&mut self, req: &DialogRequest) -> Picked<PathBuf> {
        Ok(Self::dialog(req).pick_file())
    }

    fn pick_files(&mut self, req: &DialogRequest) -> Picked<Vec<PathBuf>> {
        Ok(Self::dialog(req).pick_files())
    }

    fn save_file(&mut self, req: &DialogRequest) -> Picked<PathBuf> {
        Ok(Self::dialog(req).save_file())
    }

    fn pick_folder(&mut self, req: &DialogRequest) -> Picked<PathBuf> {
        Ok(Self::dialog(req).pick_folder())
    }
}

/// The dialogs of the current target: native on the desktop, none on
/// wasm and mobile.
pub fn default_dialogs() -> Box<dyn FileDialogs> {
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    {
        Box::new(NativeDialogs::new())
    }
    #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
    {
        Box::new(NoDialogs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_urna() -> DialogRequest {
        DialogRequest::new()
            .title("Open a .urna corpus")
            .filter("urna corpus", &["urna", "nest"])
            .directory("/corpora")
    }

    #[test]
    fn the_fake_answers_in_order_and_keeps_what_was_asked() {
        let mut dialogs = FakeDialogs::new()
            .will_pick("/corpora/cards.urna")
            .will_cancel()
            .will_pick_all(["/a.png", "/b.png"]);
        assert_eq!(dialogs.pending(), 3);

        let picked = dialogs.pick_file(&open_urna()).unwrap();
        assert_eq!(picked, Some(PathBuf::from("/corpora/cards.urna")));

        let export = DialogRequest::new()
            .title("Export media blob")
            .file_name("frame.jpg");
        assert_eq!(dialogs.save_file(&export).unwrap(), None, "cancelled");

        let many = dialogs.pick_files(&DialogRequest::new()).unwrap();
        assert_eq!(
            many,
            Some(vec![PathBuf::from("/a.png"), PathBuf::from("/b.png")])
        );
        assert_eq!(dialogs.pending(), 0);

        let asked = dialogs.take_asked();
        assert_eq!(asked.len(), 3);
        assert_eq!(asked[0].kind, DialogKind::PickFile);
        assert_eq!(
            asked[0].request.title.as_deref(),
            Some("Open a .urna corpus")
        );
        assert_eq!(asked[0].request.filters[0].name, "urna corpus");
        assert_eq!(asked[0].request.filters[0].extensions, ["urna", "nest"]);
        assert_eq!(
            asked[0].request.directory.as_deref(),
            Some(std::path::Path::new("/corpora"))
        );
        assert_eq!(asked[1].kind, DialogKind::SaveFile);
        assert_eq!(asked[1].request.file_name.as_deref(), Some("frame.jpg"));
        assert_eq!(asked[2].kind, DialogKind::PickFiles);
        assert!(dialogs.asked().is_empty(), "taken");
    }

    #[test]
    fn the_fake_with_nothing_loaded_is_a_cancel() {
        let mut dialogs = FakeDialogs::new();
        assert_eq!(dialogs.pick_file(&open_urna()).unwrap(), None);
        assert_eq!(dialogs.pick_folder(&DialogRequest::new()).unwrap(), None);
        assert_eq!(dialogs.asked().len(), 2);
        assert_eq!(dialogs.asked()[1].kind, DialogKind::PickFolder);
    }

    #[test]
    fn a_single_path_dialog_takes_the_first_and_an_empty_list_is_a_cancel() {
        let mut dialogs = FakeDialogs::new()
            .will_pick_all(["/first", "/second"])
            .will_pick_all(Vec::<PathBuf>::new());
        assert_eq!(
            dialogs.pick_file(&DialogRequest::new()).unwrap(),
            Some(PathBuf::from("/first"))
        );
        assert_eq!(dialogs.pick_files(&DialogRequest::new()).unwrap(), None);
    }

    #[test]
    fn the_fake_refuses_as_a_target_without_dialogs_does() {
        let mut dialogs = FakeDialogs::new().will_refuse();
        let err = dialogs.save_file(&DialogRequest::new()).unwrap_err();
        assert_eq!(err, DialogError::Unsupported(DialogKind::SaveFile));
        assert_eq!(
            err.to_string(),
            "save_file: no native file dialog on this target"
        );
    }

    #[test]
    fn a_shared_fake_answers_through_one_handle_and_reports_through_the_other() {
        let fake = Arc::new(Mutex::new(FakeDialogs::new().will_pick("/picked")));
        let mut page: Box<dyn FileDialogs> = Box::new(fake.clone());
        assert_eq!(
            page.pick_file(&open_urna()).unwrap(),
            Some(PathBuf::from("/picked"))
        );
        assert_eq!(page.save_file(&DialogRequest::new()).unwrap(), None);
        let asked = locked(&fake).take_asked();
        assert_eq!(asked.len(), 2);
        assert_eq!(asked[0].kind, DialogKind::PickFile);
        assert_eq!(asked[1].kind, DialogKind::SaveFile);
    }

    #[test]
    fn no_dialogs_is_unsupported_for_every_kind() {
        let mut none = NoDialogs;
        let req = open_urna();
        assert_eq!(
            none.pick_file(&req),
            Err(DialogError::Unsupported(DialogKind::PickFile))
        );
        assert_eq!(
            none.pick_files(&req),
            Err(DialogError::Unsupported(DialogKind::PickFiles))
        );
        assert_eq!(
            none.save_file(&req),
            Err(DialogError::Unsupported(DialogKind::SaveFile))
        );
        assert_eq!(
            none.pick_folder(&req),
            Err(DialogError::Unsupported(DialogKind::PickFolder))
        );
    }

    /// Building the `rfd` dialog opens nothing; its debug form shows the
    /// request went through whole. (The panel itself is opened only by a
    /// person, never by a test.)
    #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
    #[test]
    fn the_native_builder_carries_the_request() {
        let req = open_urna().file_name("cards.urna");
        let built = format!("{:?}", NativeDialogs::dialog(&req));
        assert!(built.contains("Open a .urna corpus"), "{built}");
        assert!(built.contains("urna corpus"), "{built}");
        assert!(built.contains("\"nest\""), "{built}");
        assert!(built.contains("/corpora"), "{built}");
        assert!(built.contains("cards.urna"), "{built}");
        let bare = format!("{:?}", NativeDialogs::dialog(&DialogRequest::new()));
        assert!(bare.contains("title: None"), "{bare}");
    }
}
