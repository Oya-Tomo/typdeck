//! Builds the deck from the Typst source and the notes, and rebuilds it
//! whenever one of the files it was built from changes.

use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::watch;
use typst::diag::{SourceDiagnostic, Warned};
use typst_kit::diagnostics::{self, DiagnosticFormat, termcolor};
use typst_kit::watcher::Watcher;
use typst_layout::PagedDocument;
use typst_svg::SvgOptions;

use crate::notes::{self, Note};
use crate::world::SlideWorld;

/// Everything the browser displays.
#[derive(Default)]
pub struct Deck {
    pub slides: Arc<[Slide]>,
    /// Speaker notes, indexed like `slides`.
    pub notes: Vec<Note>,
    /// Problems with the latest build. Whatever failed to build keeps its
    /// last successfully built state.
    pub errors: Vec<String>,
}

impl Deck {
    pub fn slide(&self, id: &str) -> Option<&Slide> {
        self.slides.iter().find(|slide| slide.id == id)
    }
}

#[derive(Clone)]
pub struct Slide {
    /// Identifies the slide's content, so that browsers only reload slides
    /// that changed.
    pub id: String,
    /// SVG markup that scales to its container. It keeps images such as
    /// animated GIFs in their original format.
    pub svg: Arc<str>,
}

impl Slide {
    fn new(svg: &str) -> Self {
        // Typst sizes the root element in points, which an inline style
        // overrides. Links open in a new tab to keep the presentation open.
        let svg = svg
            .replacen("<svg ", r#"<svg style="width: 100%; height: 100%" "#, 1)
            .replace("<a ", r#"<a target="_blank" rel="noopener" "#);

        let mut hasher = DefaultHasher::new();
        svg.hash(&mut hasher);
        Self {
            id: format!("{:016x}", hasher.finish()),
            svg: svg.into(),
        }
    }
}

/// The files the deck is built from.
pub struct Sources {
    pub world: SlideWorld,
    pub notes: Option<PathBuf>,
}

impl Sources {
    /// Rebuilds the deck into `deck` every time a source file changes.
    ///
    /// Only returns if watching the file system fails.
    pub fn watch(mut self, deck: &watch::Sender<Arc<Deck>>) -> Result<(), String> {
        let mut watcher = Watcher::new(None).map_err(String::from)?;
        loop {
            let next = self.build(&deck.borrow());
            deck.send_replace(Arc::new(next));
            watcher.update(self.dependencies()).map_err(String::from)?;
            watcher.wait().map_err(String::from)?;
        }
    }

    fn build(&mut self, previous: &Deck) -> Deck {
        let mut errors = Vec::new();

        self.world.reset();
        let slides = self.compile().unwrap_or_else(|error| {
            errors.push(error);
            previous.slides.clone()
        });

        let notes = match &self.notes {
            None => Vec::new(),
            Some(path) => match fs::read_to_string(path) {
                Ok(text) => notes::parse(&text),
                Err(error) => {
                    errors.push(format!("failed to read {}: {error}", path.display()));
                    previous.notes.clone()
                }
            },
        };

        if errors.is_empty() {
            eprintln!("built {} slides", slides.len());
        }
        for error in &errors {
            eprintln!("{error}");
        }

        Deck {
            slides,
            notes,
            errors,
        }
    }

    fn compile(&mut self) -> Result<Arc<[Slide]>, String> {
        let Warned { output, warnings } = typst::compile::<PagedDocument>(&self.world);
        if !warnings.is_empty() {
            eprint!("{}", self.report(&warnings));
        }

        let document = output.map_err(|errors| self.report(&errors))?;
        let options = SvgOptions::default();
        Ok(document
            .pages()
            .iter()
            .map(|page| Slide::new(&typst_svg::svg(page, &options)))
            .collect())
    }

    /// Formats diagnostics together with the source lines they point at.
    fn report(&self, diagnostics: &[SourceDiagnostic]) -> String {
        let mut out = termcolor::NoColor::new(Vec::new());
        diagnostics::emit(&mut out, &self.world, diagnostics, DiagnosticFormat::Human)
            .expect("writing to memory cannot fail");
        String::from_utf8_lossy(&out.into_inner()).into_owned()
    }

    fn dependencies(&mut self) -> Vec<PathBuf> {
        let mut paths = self.world.dependencies();
        paths.extend(self.notes.clone());
        paths
    }
}
