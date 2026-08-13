use super::super::{PNG_SIZES, rgba, svg};

/// Every checked-in PNG, beside the size that generated it.
fn checked() -> Vec<(u16, &'static [u8])> {
    vec![
        (16, include_bytes!("../../../../assets/yog-16.png")),
        (32, include_bytes!("../../../../assets/yog-32.png")),
        (48, include_bytes!("../../../../assets/yog-48.png")),
        (64, include_bytes!("../../../../assets/yog-64.png")),
        (128, include_bytes!("../../../../assets/yog-128.png")),
        (256, include_bytes!("../../../../assets/yog-256.png")),
    ]
}

/// The checked-in SVG is a *derivation*, not a hand-edit. If this fails the
/// geometry moved: re-emit with `make icon`, never patch the file.
#[test]
fn the_checked_in_svg_is_the_generated_one() {
    assert_eq!(svg(), include_str!("../../../../assets/yog.svg"));
}

/// Each checked-in PNG **decodes to exactly the pixels the generator makes**.
/// Comparing the decoded image rather than the encoded bytes is deliberate:
/// the encoder's settings — compression level, filter choice, even the encoder
/// itself — may change freely, and only the picture is the contract.
#[test]
fn every_checked_in_png_decodes_to_the_pixels_the_generator_makes() {
    for (size, file) in checked() {
        let decoded = image::load_from_memory_with_format(file, image::ImageFormat::Png)
            .unwrap_or_else(|error| panic!("yog-{size}.png will not decode: {error}"))
            .to_rgba8();
        assert_eq!(
            (decoded.width(), decoded.height()),
            (u32::from(size), u32::from(size)),
            "yog-{size}.png is the wrong size"
        );
        assert_eq!(
            decoded.into_raw(),
            rgba(size),
            "assets/yog-{size}.png has drifted from the mark"
        );
    }
}

/// The checked-in set is exactly the set the generator and the Makefile agree
/// on — no orphan file, no size emitted but never installed.
#[test]
fn the_checked_in_set_is_the_declared_set() {
    let sizes: Vec<u16> = checked().into_iter().map(|(size, _)| size).collect();
    assert_eq!(sizes, PNG_SIZES.to_vec());
}
