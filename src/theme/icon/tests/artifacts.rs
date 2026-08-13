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

/// Each checked-in PNG **decodes to the pixels the generator makes**. Comparing
/// the decoded image rather than the encoded bytes is deliberate: the encoder's
/// settings — compression level, filter choice, even the encoder itself — may
/// change freely, and only the picture is the contract.
///
/// The comparison is per-channel and admits a difference of one (bl-e492). The
/// rasterizer picks its kernels by what the CPU offers at run time, so two
/// machines round the edge of a shape differently in the last bit: this test
/// failed on every CI run yog ever had because one channel of one pixel of
/// yog-16.png came out `41` on the runner where the checked-in file holds `42`,
/// at alpha 25. Bit-exactness across CPUs is not a promise the rasterizer makes
/// and not one the icon needs; ±1 is invisible and still catches every real
/// drift, because moved geometry or a changed palette shifts whole runs of
/// pixels by far more than a bit.
const TOLERANCE: u8 = 1;

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
        let made = rgba(size);
        let file = decoded.into_raw();
        assert_eq!(
            file.len(),
            made.len(),
            "assets/yog-{size}.png has drifted from the mark"
        );
        let drifted: Vec<usize> = (0..file.len())
            .filter(|&i| file[i].abs_diff(made[i]) > TOLERANCE)
            .collect();
        assert!(
            drifted.is_empty(),
            "assets/yog-{size}.png has drifted from the mark at {} of {} channels, first at {:?}",
            drifted.len(),
            file.len(),
            drifted.first()
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
