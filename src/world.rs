//! The Typst [`World`] that compiles the slide deck from the file system.

use std::path::{Path, PathBuf};

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_kit::datetime::Time;
use typst_kit::diagnostics::DiagnosticWorld;
use typst_kit::files::{FileStore, FsRoot, SystemFiles};
use typst_kit::fonts::{self, FontStore};
use typst_kit::packages::SystemPackages;

use crate::downloader::HttpDownloader;

pub struct SlideWorld {
    library: LazyHash<Library>,
    fonts: FontStore,
    files: FileStore<SystemFiles>,
    main: FileId,
    time: Time,
}

impl SlideWorld {
    /// Creates a world for compiling `input`, resolving absolute Typst paths
    /// against `root`.
    pub fn new(input: &Path, root: &Path) -> Result<Self, String> {
        let vpath = VirtualPath::virtualize(root, input).map_err(|err| {
            format!(
                "{} must be inside the root {} ({err})",
                input.display(),
                root.display()
            )
        })?;

        let mut fonts = FontStore::new();
        fonts.extend(fonts::system());
        fonts.extend(fonts::embedded());

        let packages = SystemPackages::new(HttpDownloader::new());
        let files = SystemFiles::new(FsRoot::new(root.to_owned()), packages);

        Ok(Self {
            library: LazyHash::new(Library::default()),
            fonts,
            files: FileStore::new(files),
            main: RootedPath::new(VirtualRoot::Project, vpath).intern(),
            time: Time::system(),
        })
    }

    /// Prepares the world for the next compilation, so that changed files are
    /// read again and the current date is refreshed.
    pub fn reset(&mut self) {
        self.files.reset();
        self.time = Time::system();
    }

    /// The file system paths the last compilation read.
    pub fn dependencies(&mut self) -> Vec<PathBuf> {
        let (loader, ids) = self.files.dependencies();
        ids.filter_map(|id| loader.resolve(id).ok()).collect()
    }
}

impl World for SlideWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.files.source(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files.file(id)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        self.time.today(offset)
    }
}

impl DiagnosticWorld for SlideWorld {
    fn name(&self, id: FileId) -> String {
        self.files.loader().resolve(id).map_or_else(
            |_: FileError| format!("{id:?}"),
            |path| path.display().to_string(),
        )
    }
}
