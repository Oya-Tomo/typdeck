//! The presenter console: the current and next slide, the notes, a timer, and
//! an overview of all slides.

use std::io::ErrorKind;
use std::time::Duration;

use topcoat::{
    Result,
    context::{Cx, app_context},
    icon::{IconData, icon},
    router::{
        error::{RouterErrorExt, bad_request},
        href, page,
    },
    runtime::{Event, Signal, connected, procedure, signal},
    view::{Unescaped, View, component, emit, live, svg::ViewBox, view},
};

use super::{current_slide, slides, viewer};
use crate::deck::Slide;
use crate::notes::{self, Note};
use crate::presentation::Presentation;
use crate::stopwatch::Stopwatch;

/// Marks the slide being presented.
const ACCENT: &str = "#e5484d";

const CONSOLE: &str = "display: grid; height: 100vh; box-sizing: border-box; padding: 12px; \
    gap: 12px; grid-template: 'counter status' auto 'current side' minmax(0, 1fr) \
    'overview overview' auto / minmax(0, 3fr) minmax(0, 2fr); background: #16181d; color: #e6e6e6; \
    font-family: system-ui, sans-serif";
const STATUS: &str = "grid-area: status; display: flex; flex-wrap: wrap; align-items: center; \
    gap: 8px";
const SIDE: &str =
    "grid-area: side; display: flex; flex-direction: column; gap: 12px; min-height: 0";
const NOTES: &str = "flex: 1; overflow: auto; padding: 12px 16px; background: #23262d; \
    border-radius: 6px; font-size: 1.3rem; line-height: 1.6";
const EDITOR: &str = "box-sizing: border-box; width: 100%; height: 40vh; padding: 8px; \
    font: 1.1rem/1.5 ui-monospace, monospace; color: inherit; background: #16181d; \
    border: 1px solid #3a3f48; border-radius: 4px";
const ERRORS: &str = "margin: 0; max-height: 40vh; overflow: auto; padding: 12px; color: #fff; \
    background: #7f1d1d; border-radius: 6px; white-space: pre-wrap";
const THUMBNAIL: &str = "flex: none; width: 150px; padding: 0; border: none; background: none; \
    color: inherit; font: inherit; cursor: pointer";
/// The color of Typst's brand, for links and hovered buttons.
const TYPST: &str = "#239dad";

/// Styles the `button` class in a stylesheet, because hover styles cannot be
/// written inline.
fn button_stylesheet() -> Unescaped<String> {
    Unescaped::new_unchecked(format!(
        ".button {{ display: inline-flex; align-items: center; justify-content: center; \
            min-height: 2em; padding: 2px 10px; font: inherit; color: inherit; \
            background: #23262d; border: 1px solid #3a3f48; border-radius: 4px; cursor: pointer }} \
        .button:hover {{ color: #fff; background: {TYPST}; border-color: {TYPST} }}"
    ))
}

const START: IconData = IconData::unescaped_unchecked(
    ViewBox::new(0.0, 0.0, 24.0, 24.0),
    r#"<path fill="currentColor" d="M8 5v14l11-7z"/>"#,
);
const PAUSE: IconData = IconData::unescaped_unchecked(
    ViewBox::new(0.0, 0.0, 24.0, 24.0),
    r#"<path fill="currentColor" d="M6 5h4v14H6zm8 0h4v14h-4z"/>"#,
);
const RESET: IconData = IconData::unescaped_unchecked(
    ViewBox::new(0.0, 0.0, 24.0, 24.0),
    r#"<path fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" d="M19.5 12a7.5 7.5 0 1 1-2.2-5.3"/><path fill="currentColor" d="M21 3v7h-7z"/>"#,
);

/// Buttons that correct the timer, with the seconds they add.
const TIMER_ADJUSTMENTS: [(&str, i64); 4] =
    [("−1m", -60), ("−10s", -10), ("+10s", 10), ("+1m", 60)];

#[page]
pub async fn page(cx: &Cx) -> Result<impl View> {
    // The index of the note being edited, and its edited text.
    let editing = signal(cx, || None::<usize>);
    let draft = signal(cx, String::new);

    Ok(view! {
        <style>(button_stylesheet())</style>
        <div style=(CONSOLE)>
            <section style="grid-area: current; min-height: 0">current_slide()</section>
            // Shares the row above the current slide with the slide counter,
            // on the opposite side.
            <a
                href=(href!(viewer::page))
                target="typdeck-viewer"
                style=(format!(
                    "grid-area: counter; justify-self: end; align-self: center; \
                        margin-right: 24px; color: {TYPST}",
                ))
            >
                "Open viewer"
            </a>
            <div style=(STATUS)>timer()</div>
            console(editing: &editing, draft: &draft)
        </div>
    })
}

/// The time spent presenting, with buttons to run, reset, and correct it.
///
/// Pressing a button keeps the focus where it was, so that the space key keeps
/// moving through the slides instead of pressing the button again.
#[component]
async fn timer(cx: &Cx) -> Result<impl View> {
    Ok(live! {
        let mut timer = app_context::<Presentation>(cx).timer();
        loop {
            let stopwatch = *timer.borrow_and_update();
            let elapsed = stopwatch.elapsed();
            let seconds = elapsed.as_secs();
            let token = emit! {
                <span
                    style="min-width: 4ch; font-size: 1.5rem; font-variant-numeric: tabular-nums"
                >
                    (seconds / 60)
                    ":"
                    (format!("{:02}", seconds % 60))
                </span>
                <button
                    class="button"
                    @mousedown=$(|e: Event| e.prevent_default())
                    @click=$(async |_e| toggle_timer().await)
                >
                    if stopwatch.is_running() {
                        icon(data: PAUSE, label: "Pause")
                    } else {
                        icon(data: START, label: "Start")
                    }
                </button>
                <button
                    class="button"
                    @mousedown=$(|e: Event| e.prevent_default())
                    @click=$(async |_e| reset_timer().await)
                >
                    icon(data: RESET, label: "Reset")
                </button>
                for (label, seconds) in TIMER_ADJUSTMENTS {
                    <button
                        class="button"
                        @mousedown=$(|e: Event| e.prevent_default())
                        @click=$(async |_e| adjust_timer(seconds).await)
                    >
                        (label)
                    </button>
                }
            }?;
            if !connected(cx) {
                break Ok(token);
            }

            if stopwatch.is_running() {
                let next_second = Duration::from_secs(seconds + 1).saturating_sub(
                    elapsed,
                );
                tokio::select! {
                    () = tokio::time::sleep(next_second) => {}
                    _ = timer.changed() => {}
                }
            } else {
                timer.changed().await.ok();
            }
        }
    })
}

/// Everything that follows the current slide: the next slide, the notes, build
/// errors, and the overview.
#[component]
async fn console(
    cx: &Cx,
    editing: &Signal<Option<usize>>,
    draft: &Signal<String>,
) -> Result<impl View> {
    let editable = app_context::<Presentation>(cx).notes().is_some();
    Ok(live! {
        let mut updates = app_context::<Presentation>(cx).updates();
        loop {
            let snapshot = updates.latest();
            let index = snapshot.index;
            let slides = &snapshot.deck.slides;
            let note = snapshot.note().cloned().unwrap_or_default();

            let token = emit! {
                <div
                    style="grid-area: counter; justify-self: start; align-self: center; font-size: 1.25rem"
                >
                    "Slide "
                    (if slides.is_empty() { 0 } else { index + 1 })
                    " / "
                    (slides.len())
                </div>
                <aside style=(SIDE)>
                    <div style="color: #9aa0aa">"Next"</div>
                    match snapshot.next() {
                        Some(slide) => {
                            preview(slide: slide, style: "max-height: 30vh")
                        }
                        None => <p style="margin: 0; color: #9aa0aa">"End of slides"</p>,
                    }
                    notes_panel(
                        note: note,
                        index: index,
                        editable: editable,
                        editing: editing,
                        draft: draft
                    )
                    build_errors(errors: &snapshot.deck.errors)
                </aside>
                overview(slides: slides, current: index)
            }?;
            if !connected(cx) {
                break Ok(token);
            }
            updates.changed().await;
        }
    })
}

/// The note of the current slide, and an editor for it.
#[component]
async fn notes_panel(
    note: Note,
    index: usize,
    editable: bool,
    editing: &Signal<Option<usize>>,
    draft: &Signal<String>,
) -> Result<impl View> {
    let Note { markdown, html } = note;
    Ok(view! {
        <section style=(NOTES)>
            <div :hidden=$(editing.get().is_some())>
                // Notes are written by the presenter, so they are trusted.
                (Unescaped::new_unchecked(html))
            </div>
            if editable {
                <form
                    :hidden=$(editing.get().is_none())
                    @submit=$(async |e: Event| {
                        e.prevent_default();
                        save_note(editing.get().unwrap(), draft.get()).await;
                        editing.set(None);
                    })
                >
                    // Stopping key events keeps typing from moving through the slides.
                    <textarea
                        style=(EDITOR)
                        :value=$(draft.get())
                        @input=$(|e: Event| draft.set(e.target.value))
                        @keydown=$(|e: Event| e.stop_propagation())
                    ></textarea>
                    <button type="submit" class="button">"Save"</button>
                    " "
                    <button
                        type="button"
                        class="button"
                        @click=$(|_e| editing.set(None))
                    >
                        "Cancel"
                    </button>
                </form>
                <button
                    class="button"
                    :hidden=$(editing.get().is_some())
                    @click=$(|_e| {
                        draft.set(markdown);
                        editing.set(Some(index));
                    })
                >
                    "Edit note"
                </button>
            }
        </section>
    })
}

/// Why the latest changes to the slides or notes are not shown.
#[component]
async fn build_errors(errors: &[String]) -> Result<impl View> {
    Ok(view! {
        if !errors.is_empty() {
            <pre style=(ERRORS)>(errors.join("\n\n"))</pre>
        }
    })
}

/// All slides, with the current one outlined. Clicking a slide presents it.
#[component]
async fn overview(slides: &[Slide], current: usize) -> Result<impl View> {
    Ok(view! {
        <nav
            style="grid-area: overview; display: flex; gap: 10px; overflow-x: auto; padding: 4px"
        >
            for (i, slide) in slides.iter().enumerate() {
                // Preventing the focus on mouse down keeps the space key moving
                // through the slides instead of clicking the button again.
                <button
                    style=(format!(
                        "{THUMBNAIL}; outline: 3px solid {}",
                        if i == current { ACCENT } else { "transparent" },
                    ))
                    @mousedown=$(|e: Event| e.prevent_default())
                    @click=$(async |_e| go_to(i).await)
                >
                    preview(slide: slide, style: "")
                    (i + 1)
                </button>
            }
        </nav>
    })
}

/// A slide as an image, which does not need to be interactive.
#[component]
async fn preview(slide: &Slide, style: &str) -> Result<impl View> {
    Ok(view! {
        <img
            src=(href!(slides::slide, slides::SlideId(&slide.id)))
            alt=""
            style=(format!("display: block; width: 100%; object-fit: contain; {style}"))
        >
    })
}

#[procedure]
async fn go_to(cx: &Cx, index: usize) -> Result<()> {
    app_context::<Presentation>(cx).go_to(index);
    Ok(())
}

#[procedure]
async fn toggle_timer(cx: &Cx) -> Result<()> {
    app_context::<Presentation>(cx).update_timer(Stopwatch::toggle);
    Ok(())
}

#[procedure]
async fn reset_timer(cx: &Cx) -> Result<()> {
    app_context::<Presentation>(cx).update_timer(Stopwatch::reset);
    Ok(())
}

#[procedure]
async fn adjust_timer(cx: &Cx, seconds: i64) -> Result<()> {
    app_context::<Presentation>(cx).update_timer(|stopwatch| stopwatch.adjust(seconds));
    Ok(())
}

/// Writes the note of one slide back to the notes file. The file watcher then
/// rebuilds the deck, which updates every view.
#[procedure]
async fn save_note(cx: &Cx, index: usize, markdown: String) -> Result<()> {
    let presentation = app_context::<Presentation>(cx);
    let path = presentation.notes().ok_or_not_found()?;
    if index >= presentation.deck().slides.len() {
        return Err(bad_request("no such slide").into());
    }

    let text = match tokio::fs::read_to_string(path).await {
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        text => text?,
    };
    tokio::fs::write(path, notes::replace(&text, index, &markdown)).await?;
    Ok(())
}
