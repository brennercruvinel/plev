//! CPU-side WGSL validation: parse + validate every engine shader with naga
//! so a broken shader fails `cargo test` instead of panicking at runtime.

use wgpu::naga;

fn validate_wgsl(name: &str, source: &str) {
    let module = naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|e| panic!("{name}: WGSL parse error: {e}"));
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .unwrap_or_else(|e| panic!("{name}: WGSL validation error: {e:?}"));
}

#[test]
fn all_engine_shaders_are_valid_wgsl() {
    let shaders = [
        ("quad.wgsl", include_str!("../../shaders/quad.wgsl")),
        ("rect_sdf.wgsl", include_str!("../../shaders/rect_sdf.wgsl")),
        (
            "shadow_analytic.wgsl",
            include_str!("../../shaders/shadow_analytic.wgsl"),
        ),
        ("image.wgsl", include_str!("../../shaders/image.wgsl")),
        ("backdrop.wgsl", include_str!("../../shaders/backdrop.wgsl")),
        ("text.wgsl", include_str!("../../shaders/text.wgsl")),
        (
            "composite.wgsl",
            include_str!("../../shaders/composite.wgsl"),
        ),
        ("blur.wgsl", include_str!("../../shaders/blur.wgsl")),
        ("shadow.wgsl", include_str!("../../shaders/shadow.wgsl")),
    ];
    for (name, source) in shaders {
        validate_wgsl(name, source);
    }
}
