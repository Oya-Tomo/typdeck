mod app;
mod assets;
mod deck;
mod downloader;
mod notes;
mod presentation;
mod stopwatch;
mod world;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;
use tokio::net::TcpListener;
use tokio::sync::watch;

use crate::deck::{Deck, Sources};
use crate::presentation::Presentation;
use crate::world::SlideWorld;

/// Present Typst slides in the browser, with speaker notes and live reload.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// The Typst file containing the slides.
    slides: PathBuf,

    /// A Markdown file with speaker notes, one section per slide separated by
    /// `---` lines. It is created when a note is edited in the presenter view.
    #[arg(short, long, value_name = "FILE")]
    notes: Option<PathBuf>,

    /// The root for absolute paths in Typst [default: the slides' directory].
    #[arg(long, value_name = "DIR")]
    root: Option<PathBuf>,

    /// The address to listen on.
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// The port to listen on.
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Args::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run(args: Args) -> Result<(), String> {
    let slides = absolute(&args.slides)?;
    if !slides.is_file() {
        return Err(format!("{} is not a file", slides.display()));
    }
    let root = match &args.root {
        Some(root) => absolute(root)?,
        None => slides.parent().expect("a file has a parent").to_owned(),
    };
    let notes = args.notes.as_deref().map(absolute).transpose()?;

    let assets = tokio::task::spawn_blocking(assets::bundle)
        .await
        .expect("bundling does not panic")?;

    let sources = Sources {
        world: SlideWorld::new(&slides, &root)?,
        notes: notes.clone(),
    };
    let (deck, decks) = watch::channel(Arc::new(Deck::default()));
    std::thread::spawn(move || {
        if let Err(error) = sources.watch(&deck) {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    });

    let listener = TcpListener::bind((args.host.as_str(), args.port))
        .await
        .map_err(|error| format!("failed to listen on {}:{}: {error}", args.host, args.port))?;
    let addr = listener.local_addr().map_err(|error| error.to_string())?;
    println!("viewer:    http://{addr}/viewer");
    println!("presenter: http://{addr}/presenter");

    let router = app::router(Presentation::new(decks, notes), assets);
    topcoat::serve(listener, router)
        .await
        .map_err(|error| error.to_string())
}

fn absolute(path: &Path) -> Result<PathBuf, String> {
    std::path::absolute(path).map_err(|error| format!("invalid path {}: {error}", path.display()))
}
