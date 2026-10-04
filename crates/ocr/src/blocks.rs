use crate::{Rect, TextLine};

/// A paragraph-like group of lines that should be translated together.
#[derive(Clone, Debug, PartialEq)]
pub struct TextBlock {
    pub lines: Vec<TextLine>,
    pub bounds: Rect,
}

impl TextBlock {
    /// The block's text with lines joined by spaces (hyphenated line breaks are
    /// re-joined into one word).
    pub fn text(&self) -> String {
        let mut text = String::new();
        for line in &self.lines {
            let line = line.text.trim();
            if text.ends_with('-') && line.starts_with(|c: char| c.is_lowercase()) {
                text.pop();
            } else if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(line);
        }
        text
    }

    /// Typical line height in the block, in pixels.
    pub fn line_height(&self) -> f32 {
        let mut heights: Vec<f32> = self.lines.iter().map(|l| l.bounds.height).collect();
        heights.sort_by(f32::total_cmp);
        heights[heights.len() / 2]
    }
}

/// Group lines into blocks: a line joins the block above it when it sits right
/// below the block's last line, has a similar height and overlaps horizontally
/// with a shared left edge or centre.
pub fn group_into_blocks(mut lines: Vec<TextLine>) -> Vec<TextBlock> {
    lines.sort_by(|a, b| a.bounds.y.total_cmp(&b.bounds.y).then(a.bounds.x.total_cmp(&b.bounds.x)));

    let mut blocks: Vec<TextBlock> = Vec::new();
    for line in lines {
        let target = blocks.iter_mut().rev().find(|block| continues(block, &line));
        match target {
            Some(block) => {
                block.bounds = block.bounds.union(&line.bounds);
                block.lines.push(line);
            }
            None => blocks.push(TextBlock {
                bounds: line.bounds,
                lines: vec![line],
            }),
        }
    }
    blocks
}

fn continues(block: &TextBlock, line: &TextLine) -> bool {
    let last = &block.lines.last().expect("blocks are never empty").bounds;
    let next = &line.bounds;
    let height = last.height.max(next.height);

    let similar_height = (last.height - next.height).abs() <= 0.35 * height;
    let gap = next.y - last.bottom();
    let close_below = gap >= -0.3 * height && gap <= 0.9 * height;
    let overlaps = next.x < last.right() && last.x < next.right();
    let aligned_left = (next.x - last.x).abs() <= 1.5 * height;
    let aligned_centre = ((next.x + next.right()) - (last.x + last.right())).abs() / 2.0 <= height;

    similar_height && close_below && overlaps && (aligned_left || aligned_centre)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(text: &str, x: f32, y: f32, width: f32, height: f32) -> TextLine {
        TextLine {
            text: text.into(),
            bounds: Rect { x, y, width, height },
            confidence: 1.0,
        }
    }

    #[test]
    fn joins_paragraph_lines_and_separates_columns_and_headings() {
        let blocks = group_into_blocks(vec![
            line("Heading", 10.0, 10.0, 200.0, 40.0),
            line("First line of a para-", 10.0, 70.0, 300.0, 20.0),
            line("graph continues here", 10.0, 94.0, 280.0, 20.0),
            line("Other column", 500.0, 70.0, 200.0, 20.0),
        ]);
        let texts: Vec<_> = blocks.iter().map(TextBlock::text).collect();
        assert_eq!(
            texts,
            ["Heading", "First line of a paragraph continues here", "Other column"]
        );
    }

    #[test]
    fn splits_on_large_vertical_gap() {
        let blocks = group_into_blocks(vec![
            line("One", 10.0, 10.0, 100.0, 20.0),
            line("Two", 10.0, 80.0, 100.0, 20.0),
        ]);
        assert_eq!(blocks.len(), 2);
    }
}
