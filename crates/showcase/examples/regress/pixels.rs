//! The third question, asked only when a layer's hash moved: which
//! pixels differ, compared inside the regions of the layers that
//! changed and nowhere else, and how many of them fall inside a rect.
//!
//! A region to compare is taken generously (whole physical pixels out,
//! one more for antialiasing): comparing a little more costs little and
//! misses nothing. A rect asked about is taken exactly, by pixel center,
//! so two controls that touch never share a pixel.

use engine::headless::Frame;
use serde::Serialize;

/// What became of the pixel question for one baseline.
pub enum Pixels {
    /// No layer's hash moved: no frame was drawn.
    NotNeeded,
    /// A layer moved and the baseline keeps no image to compare with.
    NoBaselineImage,
    /// A layer moved and this machine has no GPU adapter to draw the
    /// frame (a CI runner without one, as the headless tests skip).
    NoAdapter,
    /// Compared inside the regions of the layers that moved.
    Diffed(Changed),
}

/// The pixels that differ between a baseline image and a frame, inside
/// the compared regions only.
pub struct Changed {
    width: u32,
    height: u32,
    scale: f32,
    mask: Vec<bool>,
    /// Pixels compared (each once, however many regions hold it).
    pub compared: u64,
}

/// The changed pixels inside a rect, and the smallest logical rect that
/// holds them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Hits {
    pub pixels: u64,
    pub region: Option<[f32; 4]>,
}

/// Compare `base` and `now` inside `regions` (logical px). Two frames of
/// other sizes are an error: the baseline was taken at another viewport
/// or scale.
pub fn diff(
    base: &Frame,
    now: &Frame,
    regions: &[[f32; 4]],
    scale: f32,
) -> Result<Changed, String> {
    if (base.width, base.height) != (now.width, now.height) || base.rgba.len() != now.rgba.len() {
        return Err(format!(
            "the baseline image is {}x{}, the frame is {}x{}",
            base.width, base.height, now.width, now.height
        ));
    }
    let (w, h) = (now.width, now.height);
    let mut mask = vec![false; w as usize * h as usize];
    let mut seen = vec![false; mask.len()];
    let mut compared = 0;
    for r in regions {
        let Some([x0, y0, x1, y1]) = outward(*r, scale, w, h) else {
            continue;
        };
        for y in y0..y1 {
            for x in x0..x1 {
                let at = y as usize * w as usize + x as usize;
                if seen[at] {
                    continue;
                }
                seen[at] = true;
                compared += 1;
                let i = at * 4;
                mask[at] = base.rgba.get(i..i + 4) != now.rgba.get(i..i + 4);
            }
        }
    }
    Ok(Changed {
        width: w,
        height: h,
        scale,
        mask,
        compared,
    })
}

impl Changed {
    /// Every changed pixel.
    pub fn total(&self) -> Hits {
        self.within_except([0.0, 0.0, f32::MAX, f32::MAX], &[])
    }

    /// The changed pixels whose center lies in `rect`.
    pub fn within(&self, rect: [f32; 4]) -> Hits {
        self.within_except(rect, &[])
    }

    /// The changed pixels whose center lies in `rect` and in none of
    /// `holes` (the part of a layer no bound control covers).
    pub fn within_except(&self, rect: [f32; 4], holes: &[[f32; 4]]) -> Hits {
        let Some([x0, y0, x1, y1]) = centers(rect, self.scale, self.width, self.height) else {
            return Hits::default();
        };
        let holes: Vec<[u32; 4]> = holes
            .iter()
            .filter_map(|h| centers(*h, self.scale, self.width, self.height))
            .collect();
        let mut pixels = 0;
        let mut lo = [u32::MAX; 2];
        let mut hi = [0u32; 2];
        for y in y0..y1 {
            for x in x0..x1 {
                if !self.mask[y as usize * self.width as usize + x as usize] {
                    continue;
                }
                if holes
                    .iter()
                    .any(|h| x >= h[0] && x < h[2] && y >= h[1] && y < h[3])
                {
                    continue;
                }
                pixels += 1;
                lo = [lo[0].min(x), lo[1].min(y)];
                hi = [hi[0].max(x + 1), hi[1].max(y + 1)];
            }
        }
        let s = self.scale;
        let region = (pixels > 0).then(|| {
            [
                lo[0] as f32 / s,
                lo[1] as f32 / s,
                (hi[0] - lo[0]) as f32 / s,
                (hi[1] - lo[1]) as f32 / s,
            ]
        });
        Hits { pixels, region }
    }
}

/// `r` in physical px, out to whole pixels and one more, clamped:
/// `[x0, y0, x1, y1)`.
fn outward(r: [f32; 4], scale: f32, w: u32, h: u32) -> Option<[u32; 4]> {
    let side = |lo: f32, hi: f32, max: u32| {
        let a = ((lo * scale).floor() - 1.0).max(0.0);
        let b = ((hi * scale).ceil() + 1.0).min(max as f32);
        (b > a).then_some((a as u32, b as u32))
    };
    let (x0, x1) = side(r[0], r[0] + r[2], w)?;
    let (y0, y1) = side(r[1], r[1] + r[3], h)?;
    Some([x0, y0, x1, y1])
}

/// The pixels of `r` by center: pixel `x` is in when `x + 0.5` lies in
/// `[lo, hi)` in physical px. `[x0, y0, x1, y1)`.
fn centers(r: [f32; 4], scale: f32, w: u32, h: u32) -> Option<[u32; 4]> {
    let side = |lo: f32, hi: f32, max: u32| {
        let a = (lo * scale - 0.5).ceil().clamp(0.0, max as f32);
        let b = (hi * scale - 0.5).ceil().clamp(0.0, max as f32);
        (b > a).then_some((a as u32, b as u32))
    };
    let (x0, x1) = side(r[0], r[0] + r[2], w)?;
    let (y0, y1) = side(r[1], r[1] + r[3], h)?;
    Some([x0, y0, x1, y1])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(w: u32, h: u32) -> Frame {
        Frame {
            width: w,
            height: h,
            rgba: vec![30; (w * h * 4) as usize],
        }
    }

    /// Paint the physical block `[x0, x1) x [y0, y1)`.
    fn paint(f: &mut Frame, x0: u32, y0: u32, x1: u32, y1: u32) {
        for y in y0..y1 {
            for x in x0..x1 {
                let i = ((y * f.width + x) * 4) as usize;
                f.rgba[i] = 200;
            }
        }
    }

    fn changed(base: &Frame, now: &Frame, regions: &[[f32; 4]], scale: f32) -> Changed {
        match diff(base, now, regions, scale) {
            Ok(c) => c,
            Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn only_the_regions_asked_for_are_compared() {
        let base = frame(100, 60);
        let mut now = frame(100, 60);
        paint(&mut now, 10, 10, 20, 20);
        paint(&mut now, 70, 40, 80, 50);
        let c = changed(&base, &now, &[[0.0, 0.0, 40.0, 30.0]], 1.0);
        // the second block is outside the region: never looked at
        assert_eq!(c.total().pixels, 100);
        assert_eq!(c.total().region, Some([10.0, 10.0, 10.0, 10.0]));
        // the region is compared out to whole pixels and one more
        assert_eq!(c.compared, 41 * 31);
    }

    #[test]
    fn a_rect_counts_by_pixel_center_so_touching_rows_share_nothing() {
        let base = frame(100, 100);
        let mut now = frame(100, 100);
        // everything from row 60 down: the second of two rows 44 px
        // tall from 16.2, the todo's rows
        paint(&mut now, 0, 60, 100, 100);
        let c = changed(&base, &now, &[[0.0, 0.0, 100.0, 100.0]], 1.0);
        let first = c.within([0.0, 16.2, 100.0, 44.0]);
        let second = c.within([0.0, 60.2, 100.0, 44.0]);
        assert_eq!(first.pixels, 0, "{first:?}");
        assert_eq!(second.pixels, 100 * 40);
        assert_eq!(second.region, Some([0.0, 60.0, 100.0, 40.0]));
    }

    #[test]
    fn a_hole_takes_its_pixels_out_and_the_scale_maps_back_to_logical() {
        let base = frame(200, 200);
        let mut now = frame(200, 200);
        paint(&mut now, 20, 20, 40, 40);
        paint(&mut now, 120, 120, 140, 140);
        let c = changed(&base, &now, &[[0.0, 0.0, 100.0, 100.0]], 2.0);
        let rest = c.within_except([0.0, 0.0, 100.0, 100.0], &[[5.0, 5.0, 20.0, 20.0]]);
        assert_eq!(rest.pixels, 400);
        assert_eq!(rest.region, Some([60.0, 60.0, 10.0, 10.0]));
        assert_eq!(
            c.within([10.0, 10.0, 10.0, 10.0]).region,
            Some([10.0, 10.0, 10.0, 10.0])
        );
    }

    #[test]
    fn frames_of_other_sizes_are_an_error_and_equal_frames_have_no_change() {
        match diff(
            &frame(10, 10),
            &frame(20, 10),
            &[[0.0, 0.0, 10.0, 10.0]],
            1.0,
        ) {
            Err(e) => assert!(e.contains("10x10") && e.contains("20x10"), "{e}"),
            Ok(_) => panic!("frames of two sizes compared"),
        }
        let same = changed(
            &frame(10, 10),
            &frame(10, 10),
            &[[0.0, 0.0, 10.0, 10.0]],
            1.0,
        );
        assert_eq!(same.total(), Hits::default());
        assert_eq!(same.compared, 100);
    }
}
