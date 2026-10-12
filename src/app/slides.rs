//! Slides as standalone SVG files, for previews.

use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{error::RouterErrorExt, header, path_param, route},
};

use crate::presentation::Presentation;

path_param!(pub slide_id);

/// Serves a slide by its content ID, which makes the response immutable.
#[route(GET "./{slide_id}")]
pub async fn slide(cx: &Cx) -> Result<([(header::HeaderName, &'static str); 2], String)> {
    let deck = app_context::<Presentation>(cx).deck();
    let slide = deck.slide(path_param::<SlideId>(cx)).ok_or_not_found()?;
    Ok((
        [
            (header::CONTENT_TYPE, "image/svg+xml"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        slide.svg.to_string(),
    ))
}
