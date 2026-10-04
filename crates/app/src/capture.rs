use anyhow::Context as _;
use image::RgbaImage;

/// A frozen picture of one display.
pub struct Screenshot {
    pub image: RgbaImage,
    /// Physical pixels per logical pixel.
    pub scale: f32,
}

/// Capture the display at the given global logical position.
pub fn capture_display_at(x: f32, y: f32) -> anyhow::Result<Screenshot> {
    let monitor =
        xcap::Monitor::from_point(x as i32, y as i32).context("finding display under cursor")?;
    let image = monitor.capture_image().context("capturing screen")?;
    let logical_width = monitor.width().context("reading display width")? as f32;
    Ok(Screenshot {
        scale: image.width() as f32 / logical_width,
        image,
    })
}
