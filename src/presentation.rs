//! The state of a running presentation, shared by every open view.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::watch;

use crate::deck::{Deck, Slide};
use crate::notes::Note;
use crate::stopwatch::Stopwatch;

pub struct Presentation {
    deck: watch::Receiver<Arc<Deck>>,
    /// The index of the slide being presented.
    position: watch::Sender<usize>,
    timer: watch::Sender<Stopwatch>,
    notes: Option<PathBuf>,
}

impl Presentation {
    pub fn new(deck: watch::Receiver<Arc<Deck>>, notes: Option<PathBuf>) -> Self {
        Self {
            deck,
            position: watch::Sender::new(0),
            timer: watch::Sender::default(),
            notes,
        }
    }

    pub fn deck(&self) -> Arc<Deck> {
        self.deck.borrow().clone()
    }

    /// The notes file, if notes were given.
    pub fn notes(&self) -> Option<&Path> {
        self.notes.as_deref()
    }

    /// Subscribes to changes of the deck and the position.
    pub fn updates(&self) -> Updates {
        Updates {
            deck: self.deck.clone(),
            position: self.position.subscribe(),
        }
    }

    /// Moves to the slide at `index`, or the last slide if there are fewer.
    pub fn go_to(&self, index: usize) {
        let last = self.last_index();
        self.position.send_replace(index.min(last));
    }

    /// Moves `offset` slides forward, or backward if negative.
    pub fn step(&self, offset: isize) {
        let last = self.last_index();
        self.position
            .send_modify(|index| *index = index.saturating_add_signed(offset).min(last));
    }

    fn last_index(&self) -> usize {
        self.deck.borrow().slides.len().saturating_sub(1)
    }

    /// Subscribes to the presenter's timer.
    pub fn timer(&self) -> watch::Receiver<Stopwatch> {
        self.timer.subscribe()
    }

    pub fn update_timer(&self, update: impl FnOnce(&mut Stopwatch)) {
        self.timer.send_modify(update);
    }
}

/// A subscription to the deck and the position.
pub struct Updates {
    deck: watch::Receiver<Arc<Deck>>,
    position: watch::Receiver<usize>,
}

impl Updates {
    /// The current state, which later changes are compared against.
    pub fn latest(&mut self) -> Snapshot {
        let deck = self.deck.borrow_and_update().clone();
        let position = *self.position.borrow_and_update();
        let index = position.min(deck.slides.len().saturating_sub(1));
        Snapshot { deck, index }
    }

    /// Waits until the deck or the position changes.
    pub async fn changed(&mut self) {
        // Both senders live as long as the application, so neither fails.
        tokio::select! {
            _ = self.deck.changed() => {}
            _ = self.position.changed() => {}
        }
    }
}

/// The deck and the slide being presented at one point in time.
pub struct Snapshot {
    pub deck: Arc<Deck>,
    /// The index of the current slide, within the deck's bounds.
    pub index: usize,
}

impl Snapshot {
    pub fn current(&self) -> Option<&Slide> {
        self.deck.slides.get(self.index)
    }

    pub fn next(&self) -> Option<&Slide> {
        self.deck.slides.get(self.index + 1)
    }

    pub fn note(&self) -> Option<&Note> {
        self.deck.notes.get(self.index)
    }
}
