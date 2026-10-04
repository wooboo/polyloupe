//! cargo run -p polyloupe-ocr --release --example ocr -- screenshot.png
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    let path = std::env::args().nth(1).expect("usage: ocr <image>");
    let image = image::open(path)?.to_rgba8();
    let t = Instant::now();
    let lines = polyloupe_ocr::recognize(&image)?;
    println!("{} lines in {:?}", lines.len(), t.elapsed());
    for l in &lines {
        let b = l.bounds;
        println!(
            "  line [{:.0},{:.0} {:.0}x{:.0}] {}",
            b.x, b.y, b.width, b.height, l.text
        );
    }
    for block in polyloupe_ocr::group_into_blocks(lines) {
        let b = block.bounds;
        println!(
            "[{:.0},{:.0} {:.0}x{:.0}] {}",
            b.x,
            b.y,
            b.width,
            b.height,
            block.text()
        );
    }
    Ok(())
}
