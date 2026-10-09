//! The explorer's file dialogs: the Browse… open and the blob export ask
//! the platform through `engine::platform::files` (native on the desktop,
//! none on wasm and mobile, the fake in tests) with the title, filter and
//! file name the urnaui always used. Split from `explorer.rs` by
//! responsibility: that file is frozen past the file-hygiene limit and
//! may only shrink.

use engine::widgets::prelude::ToastManager;
use engine::runtime::platform::files::DialogRequest;
use engine::styling::theme::{Intent, Theme};

use super::Explorer;
use crate::backend::workers::messages::UrnaCommand;

impl Explorer {
    /// The Browse… button: the platform's open dialog, filtered to the
    /// corpus extensions; what it picks opens, a cancel does nothing, and
    /// a target without dialogs (mobile) says so.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn pick_file(&mut self, toasts: &mut ToastManager, theme: &Theme) {
        let req = DialogRequest::new()
            .title("Open a .urna corpus")
            .filter("urna corpus", &["urna", "nest"]);
        match self.dialogs.pick_file(&req) {
            Ok(Some(path)) => self.open_database(path),
            Ok(None) => {}
            Err(e) => {
                log::info!("file picker: {e}");
                toasts.push(
                    "the file picker needs the desktop app",
                    Intent::Informational,
                    theme,
                );
            }
        }
    }

    /// Export blob `index` to a path the user picks (the platform's save
    /// dialog), named after the blob's own uri. Where there is no dialog
    /// (wasm, mobile) the toast says the export needs the desktop app.
    pub(super) fn export_blob(
        &mut self,
        index: usize,
        toasts: &mut ToastManager,
        theme: &Theme,
    ) -> bool {
        let Some(db) = &self.db else {
            return false;
        };
        let Some(blob) = db.inspect.blobs.get(index) else {
            return false;
        };
        // media:// uris are relative names by construction; the final
        // component is the suggested file name.
        let name = blob
            .original_uri
            .trim_start_matches("media://")
            .rsplit('/')
            .next()
            .unwrap_or("blob")
            .to_string();
        let req = DialogRequest::new()
            .title("Export media blob")
            .file_name(&name);
        let dest = match self.dialogs.save_file(&req) {
            Ok(Some(dest)) => dest,
            Ok(None) => return true,
            Err(e) => {
                log::info!("blob export: {e}");
                toasts.push(
                    "blob export needs the desktop app",
                    Intent::Informational,
                    theme,
                );
                return true;
            }
        };
        toasts.push(
            format!("exporting {name}… (hash-verified before writing)"),
            Intent::Informational,
            theme,
        );
        self.worker.send(UrnaCommand::ExportBlob { index, dest });
        true
    }
}
