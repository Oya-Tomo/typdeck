//! The web interface: a bare viewer for the audience and a presenter console.

mod presenter;
mod slides;
mod viewer;

use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::{Cx, app_context},
    router::{Router, RouterBuilderDiscoverExt, Slot, layout, module_router},
    runtime::{Event, RouterBuilderRuntimeExt, connected, procedure},
    view::{Unescaped, View, component, emit, live, view},
};

use crate::presentation::Presentation;

pub fn router(presentation: Presentation, assets: AssetBundle) -> Router {
    module_router!()
        .discover()
        .assets(assets)
        .app_context(presentation)
        .runtime()
        .build()
}

#[layout]
async fn shell(slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <title>"typdeck"</title>
                topcoat::runtime::script()
            </head>
            // Every view controls the presentation with the keyboard.
            <body
                style="margin: 0; height: 100vh; overflow: hidden"
                @keydown=$(async |e: Event| navigate(e.key).await)
            >
                (slot)
            </body>
        </html>
    })
}

/// The slide being presented, inlined so that its links and animations work.
#[component]
async fn current_slide(cx: &Cx) -> Result<impl View> {
    Ok(live! {
        let mut updates = app_context::<Presentation>(cx).updates();
        loop {
            let slide = updates.latest().current().cloned();
            let token = emit! {
                <div style="width: 100%; height: 100%">
                    if let Some(slide) = &slide {
                        // Typst renders the SVG, so it is trusted.
                        (Unescaped::new_unchecked(slide.svg.to_string()))
                    }
                </div>
            }?;
            if !connected(cx) {
                break Ok(token);
            }

            // Moving between slides with the same content keeps animations
            // running.
            let shown = slide.map(|slide| slide.id);
            while updates.latest().current().map(|slide| &slide.id) == shown.as_ref() {
                updates.changed().await;
            }
        }
    })
}

/// Maps a key pressed in any view to a movement through the slides.
#[procedure]
async fn navigate(cx: &Cx, key: String) -> Result<()> {
    let presentation = app_context::<Presentation>(cx);
    match key.as_str() {
        "ArrowRight" | "ArrowDown" | "PageDown" | " " => presentation.step(1),
        "ArrowLeft" | "ArrowUp" | "PageUp" | "Backspace" => presentation.step(-1),
        "Home" => presentation.go_to(0),
        "End" => presentation.go_to(usize::MAX),
        _ => {}
    }
    Ok(())
}
