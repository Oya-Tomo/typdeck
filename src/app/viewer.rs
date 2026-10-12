//! The slides alone, for the audience.

use topcoat::{
    Result,
    router::page,
    view::{View, view},
};

use super::current_slide;

#[page]
pub async fn page() -> Result<impl View> {
    Ok(view! {
        <main style="width: 100vw; height: 100vh; background: black">
            current_slide()
        </main>
    })
}
