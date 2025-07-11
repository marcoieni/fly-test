use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route(
        "/",
        get(|| async {
            start_rustwide();
        }),
    );
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn start_rustwide() {
    std::thread::spawn(|| {
        run_rustwide().unwrap();
    })
    .join()
    .expect("Thread panicked")
}

use rustwide::cmd::SandboxImage;
use rustwide::{cmd::SandboxBuilder, Crate, Toolchain, WorkspaceBuilder};
use std::error::Error;
use std::path::Path;

fn run_rustwide() -> Result<(), Box<dyn Error>> {
    setup_logs();

    // Create a new workspace in .workspaces/docs-builder
    let workspace =
        WorkspaceBuilder::new(Path::new(".workspaces/docs-builder"), "rustwide-examples")
            .sandbox_image(SandboxImage::remote(
                "ghcr.io/rust-lang/crates-build-env/linux-micro",
            )?)
            .init()?;

    // Run the builds on stable
    let toolchain = Toolchain::dist("stable");
    toolchain.install(&workspace)?;

    // Fetch lazy_static from crates.io
    let krate = Crate::crates_io("lazy_static", "1.0.0");
    krate.fetch(&workspace)?;

    // Configure a sandbox with 1GB of RAM and no network access
    let sandbox = SandboxBuilder::new()
        .memory_limit(Some(1024 * 1024 * 1024))
        .enable_networking(false);

    let mut build_dir = workspace.build_dir("docs");
    build_dir.build(&toolchain, &krate, sandbox).run(|build| {
        build.cargo().args(&["doc", "--no-deps"]).run()?;
        Ok(())
    })?;

    let files_in_build_dir = std::fs::read_dir(".workspaces/docs-builder/docs/target/doc")?;
    for entry in files_in_build_dir {
        let path = entry?.path();
        if path.is_file() {
            println!("File in build directory: {}", path.display());
        }
    }

    Ok(())
}

fn setup_logs() {
    let mut env = env_logger::Builder::new();
    env.filter_module("rustwide", log::LevelFilter::Info);
    if let Ok(content) = std::env::var("RUST_LOG") {
        env.parse_filters(&content);
    }
    rustwide::logging::init_with(env.build());
}
