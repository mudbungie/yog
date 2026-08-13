//! Emits the icon artifacts — the derivation behind everything in `assets/`
//! (DESIGN §11). `make icon` runs this; nothing else may write those files.
//!
//! The PNG encoder lives here, in a **dev-dependency**, rather than in the
//! library: encoding is a build-time concern, so the shipped binary never
//! links a codec while the artifacts still get real compression.
//!
//! Takes the directory to write into, so the generator names no path of its
//! own: `cargo run --example icon -- assets`.

use image::ImageEncoder as _;

fn main() -> std::io::Result<()> {
    let into = std::path::PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "assets".into()));
    std::fs::write(into.join("yog.svg"), yog::theme::icon::svg())?;
    for size in yog::theme::icon::PNG_SIZES {
        let mut out = Vec::new();
        image::codecs::png::PngEncoder::new_with_quality(
            &mut out,
            image::codecs::png::CompressionType::Best,
            image::codecs::png::FilterType::Adaptive,
        )
        .write_image(
            &yog::theme::icon::rgba(size),
            u32::from(size),
            u32::from(size),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(std::io::Error::other)?;
        std::fs::write(into.join(format!("yog-{size}.png")), out)?;
    }
    Ok(())
}
