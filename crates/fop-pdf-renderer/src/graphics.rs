//! PDF graphics state machine
//!
//! Manages the graphics state stack (q/Q operators), current transformation
//! matrix (CTM), colors, line properties, path construction, and the current
//! clipping region.

use std::sync::Arc;

/// A 3×3 transformation matrix stored as [a b c d e f]
/// | a b 0 |
/// | c d 0 |
/// | e f 1 |
#[derive(Debug, Clone, Copy)]
pub struct Matrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Matrix {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Multiply: self * other
    pub fn concat(&self, other: &Matrix) -> Matrix {
        Matrix {
            a: self.a * other.a + self.b * other.c,
            b: self.a * other.b + self.b * other.d,
            c: self.c * other.a + self.d * other.c,
            d: self.c * other.b + self.d * other.d,
            e: self.e * other.a + self.f * other.c + other.e,
            f: self.e * other.b + self.f * other.d + other.f,
        }
    }

    /// Transform a point (x, y) by this matrix
    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }
}

/// RGBA color (values 0.0–1.0)
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn gray(g: f32) -> Self {
        Self {
            r: g,
            g,
            b: g,
            a: 1.0,
        }
    }

    pub fn to_tiny_skia_color(self) -> tiny_skia::Color {
        tiny_skia::Color::from_rgba(self.r, self.g, self.b, self.a)
            .unwrap_or(tiny_skia::Color::BLACK)
    }
}

/// Line cap style
#[derive(Debug, Clone, Copy, Default)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

/// Line join style
#[derive(Debug, Clone, Copy, Default)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

/// A single clipping outline (PDF `W`/`W*`), already mapped to screen space.
///
/// The path stores device-independent screen-space points — i.e. the current
/// transformation matrix and the page Y-flip have already been applied, but the
/// DPI scale has *not*. This mirrors how [`crate::content::DrawCommand`] paths
/// are stored, so the rasterizer applies the same DPI transform to both the
/// painted geometry and the clip mask.
#[derive(Debug, Clone)]
pub struct ClipPath {
    /// Clip outline in screen-space points (CTM applied, Y-flipped, pre-DPI).
    pub path: tiny_skia::Path,
    /// `true` ⇒ even-odd rule (`W*`); `false` ⇒ nonzero winding rule (`W`).
    pub even_odd: bool,
}

/// The accumulated clipping region of a graphics state.
///
/// In PDF, clipping is the *intersection* of every clip path established since
/// the page (or the enclosing `q`) began, and the clip is part of the graphics
/// state — so `q` saves it and `Q` restores it. The outlines are kept behind an
/// [`Arc`] so cloning a [`GraphicsState`] (on `q`) and attaching the active clip
/// to each draw command are both O(1); a new clip is appended copy-on-write.
#[derive(Debug, Clone, Default)]
pub struct ClipState {
    /// `None` ⇒ no clip (the whole page is visible). `Some` ⇒ the rendered
    /// region is the intersection of every outline in the vector.
    paths: Option<Arc<Vec<ClipPath>>>,
}

impl ClipState {
    /// Returns `true` when no clip is active (the whole page is paintable).
    pub fn is_unclipped(&self) -> bool {
        self.paths.is_none()
    }

    /// The accumulated clip outlines whose intersection forms the region.
    pub fn paths(&self) -> &[ClipPath] {
        match &self.paths {
            Some(v) => v.as_slice(),
            None => &[],
        }
    }

    /// Intersect a new clip outline into this region (copy-on-write).
    pub fn intersect(&mut self, clip: ClipPath) {
        let mut next: Vec<ClipPath> = self.paths().to_vec();
        next.push(clip);
        self.paths = Some(Arc::new(next));
    }

    /// Whether two clip states refer to the *same* underlying region, by
    /// pointer identity. Used by the rasterizer to reuse a built clip mask
    /// across consecutive draw commands without rebuilding it.
    pub fn same_region(&self, other: &ClipState) -> bool {
        match (&self.paths, &other.paths) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

/// One graphics state level
#[derive(Debug, Clone)]
pub struct GraphicsState {
    /// Current transformation matrix
    pub ctm: Matrix,
    /// Fill color
    pub fill_color: Color,
    /// Stroke color
    pub stroke_color: Color,
    /// Line width in user space
    pub line_width: f32,
    /// Line cap
    pub line_cap: LineCap,
    /// Line join
    pub line_join: LineJoin,
    /// Miter limit
    pub miter_limit: f32,
    /// Dash pattern: (lengths, phase)
    pub dash_pattern: (Vec<f32>, f32),
    /// Text rendering mode (for clipping)
    pub text_rendering_mode: u8,
    /// Current clipping region (intersection of `W`/`W*` outlines). Part of the
    /// graphics state, so `q` saves it and `Q` restores it.
    pub clip: ClipState,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            ctm: Matrix::identity(),
            fill_color: Color::BLACK,
            stroke_color: Color::BLACK,
            line_width: 1.0,
            line_cap: LineCap::default(),
            line_join: LineJoin::default(),
            miter_limit: 10.0,
            dash_pattern: (Vec::new(), 0.0),
            text_rendering_mode: 0,
            clip: ClipState::default(),
        }
    }
}

/// Graphics state stack
pub struct GraphicsStateStack {
    stack: Vec<GraphicsState>,
}

impl Default for GraphicsStateStack {
    fn default() -> Self {
        Self {
            stack: vec![GraphicsState::default()],
        }
    }
}

impl GraphicsStateStack {
    pub fn current(&self) -> &GraphicsState {
        self.stack.last().expect("graphics state stack empty")
    }

    pub fn current_mut(&mut self) -> &mut GraphicsState {
        self.stack.last_mut().expect("graphics state stack empty")
    }

    /// q — push a copy of the current state
    pub fn push(&mut self) {
        let state = self.current().clone();
        self.stack.push(state);
    }

    /// Q — pop the top state
    pub fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// Set the CTM (cm operator): pre-multiply
    pub fn concat_ctm(&mut self, m: Matrix) {
        let current_ctm = self.current().ctm;
        self.current_mut().ctm = m.concat(&current_ctm);
    }
}

/// A path segment
#[derive(Debug, Clone)]
pub enum PathSegment {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    CurveTo(f32, f32, f32, f32, f32, f32),
    Close,
    Rect(f32, f32, f32, f32),
}

/// Current path under construction
#[derive(Debug, Default, Clone)]
pub struct CurrentPath {
    pub segments: Vec<PathSegment>,
    pub current_x: f32,
    pub current_y: f32,
}

impl CurrentPath {
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.segments.push(PathSegment::MoveTo(x, y));
        self.current_x = x;
        self.current_y = y;
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.segments.push(PathSegment::LineTo(x, y));
        self.current_x = x;
        self.current_y = y;
    }

    pub fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.segments
            .push(PathSegment::CurveTo(x1, y1, x2, y2, x3, y3));
        self.current_x = x3;
        self.current_y = y3;
    }

    pub fn close(&mut self) {
        self.segments.push(PathSegment::Close);
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.segments.push(PathSegment::Rect(x, y, w, h));
    }

    pub fn clear(&mut self) {
        self.segments.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Convert to tiny_skia PathBuilder, applying a transform matrix
    pub fn to_skia_path(&self, transform: &Matrix) -> Option<tiny_skia::Path> {
        let mut pb = tiny_skia::PathBuilder::new();
        let mut has_content = false;

        for seg in &self.segments {
            match *seg {
                PathSegment::MoveTo(x, y) => {
                    let (tx, ty) = transform.transform_point(x, y);
                    pb.move_to(tx, ty);
                    has_content = true;
                }
                PathSegment::LineTo(x, y) => {
                    let (tx, ty) = transform.transform_point(x, y);
                    pb.line_to(tx, ty);
                    has_content = true;
                }
                PathSegment::CurveTo(x1, y1, x2, y2, x3, y3) => {
                    let (tx1, ty1) = transform.transform_point(x1, y1);
                    let (tx2, ty2) = transform.transform_point(x2, y2);
                    let (tx3, ty3) = transform.transform_point(x3, y3);
                    pb.cubic_to(tx1, ty1, tx2, ty2, tx3, ty3);
                    has_content = true;
                }
                PathSegment::Close => {
                    pb.close();
                }
                PathSegment::Rect(x, y, w, h) => {
                    let (tx, ty) = transform.transform_point(x, y);
                    let (tx2, ty2) = transform.transform_point(x + w, y + h);
                    // Build rect from transformed corners
                    let (rx, ry) = (tx.min(tx2), ty.min(ty2));
                    let (rw, rh) = ((tx - tx2).abs(), (ty - ty2).abs());
                    pb.push_rect(tiny_skia::Rect::from_xywh(rx, ry, rw, rh)?);
                    has_content = true;
                }
            }
        }

        if has_content {
            pb.finish()
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Matrix
    // -----------------------------------------------------------------------

    #[test]
    fn test_matrix_identity() {
        let m = Matrix::identity();
        assert_eq!(m.a, 1.0);
        assert_eq!(m.b, 0.0);
        assert_eq!(m.c, 0.0);
        assert_eq!(m.d, 1.0);
        assert_eq!(m.e, 0.0);
        assert_eq!(m.f, 0.0);
    }

    #[test]
    fn test_matrix_transform_point_with_identity() {
        let m = Matrix::identity();
        let (x, y) = m.transform_point(3.0, 4.0);
        assert!((x - 3.0).abs() < 1e-6);
        assert!((y - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_matrix_transform_point_with_translation() {
        let m = Matrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 10.0,
            f: 20.0,
        };
        let (x, y) = m.transform_point(5.0, 5.0);
        assert!((x - 15.0).abs() < 1e-6);
        assert!((y - 25.0).abs() < 1e-6);
    }

    #[test]
    fn test_matrix_transform_point_with_scale() {
        let m = Matrix {
            a: 2.0,
            b: 0.0,
            c: 0.0,
            d: 3.0,
            e: 0.0,
            f: 0.0,
        };
        let (x, y) = m.transform_point(5.0, 4.0);
        assert!((x - 10.0).abs() < 1e-6);
        assert!((y - 12.0).abs() < 1e-6);
    }

    #[test]
    fn test_matrix_transform_point_origin_unchanged() {
        let m = Matrix {
            a: 5.0,
            b: 2.0,
            c: 3.0,
            d: 7.0,
            e: 0.0,
            f: 0.0,
        };
        let (x, y) = m.transform_point(0.0, 0.0);
        assert!((x - 0.0).abs() < 1e-6);
        assert!((y - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_matrix_concat_identity_is_noop() {
        let m = Matrix {
            a: 2.0,
            b: 1.0,
            c: 0.0,
            d: 3.0,
            e: 5.0,
            f: 7.0,
        };
        let id = Matrix::identity();
        let result = m.concat(&id);
        assert!((result.a - m.a).abs() < 1e-6);
        assert!((result.d - m.d).abs() < 1e-6);
        assert!((result.e - m.e).abs() < 1e-6);
        assert!((result.f - m.f).abs() < 1e-6);
    }

    #[test]
    fn test_matrix_concat_translation() {
        // t1 = translate(10, 20), t2 = translate(5, 3)
        // Combined should translate by (15, 23)
        let t1 = Matrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 10.0,
            f: 20.0,
        };
        let t2 = Matrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 5.0,
            f: 3.0,
        };
        let combined = t1.concat(&t2);
        let (x, y) = combined.transform_point(0.0, 0.0);
        assert!((x - 15.0).abs() < 1e-5, "x = {}", x);
        assert!((y - 23.0).abs() < 1e-5, "y = {}", y);
    }

    // -----------------------------------------------------------------------
    // Color
    // -----------------------------------------------------------------------

    #[test]
    fn test_color_black_constant() {
        let c = Color::BLACK;
        assert_eq!(c.r, 0.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_color_white_constant() {
        let c = Color::WHITE;
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 1.0);
        assert_eq!(c.b, 1.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_color_rgb_constructor() {
        let c = Color::rgb(0.5, 0.25, 0.75);
        assert!((c.r - 0.5).abs() < 1e-6);
        assert!((c.g - 0.25).abs() < 1e-6);
        assert!((c.b - 0.75).abs() < 1e-6);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_color_gray_constructor() {
        let c = Color::gray(0.5);
        assert!((c.r - 0.5).abs() < 1e-6);
        assert!((c.g - 0.5).abs() < 1e-6);
        assert!((c.b - 0.5).abs() < 1e-6);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn test_color_gray_zero_is_black() {
        let c = Color::gray(0.0);
        assert_eq!(c.r, 0.0);
        assert_eq!(c.g, 0.0);
        assert_eq!(c.b, 0.0);
    }

    #[test]
    fn test_color_gray_one_is_white() {
        let c = Color::gray(1.0);
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 1.0);
        assert_eq!(c.b, 1.0);
    }

    // -----------------------------------------------------------------------
    // GraphicsState defaults
    // -----------------------------------------------------------------------

    #[test]
    fn test_graphics_state_default_fill_color_is_black() {
        let gs = GraphicsState::default();
        assert_eq!(gs.fill_color.r, 0.0);
        assert_eq!(gs.fill_color.g, 0.0);
        assert_eq!(gs.fill_color.b, 0.0);
    }

    #[test]
    fn test_graphics_state_default_stroke_color_is_black() {
        let gs = GraphicsState::default();
        assert_eq!(gs.stroke_color.r, 0.0);
        assert_eq!(gs.stroke_color.g, 0.0);
        assert_eq!(gs.stroke_color.b, 0.0);
    }

    #[test]
    fn test_graphics_state_default_line_width_is_one() {
        let gs = GraphicsState::default();
        assert!((gs.line_width - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_graphics_state_default_ctm_is_identity() {
        let gs = GraphicsState::default();
        assert_eq!(gs.ctm.a, 1.0);
        assert_eq!(gs.ctm.d, 1.0);
        assert_eq!(gs.ctm.e, 0.0);
        assert_eq!(gs.ctm.f, 0.0);
    }

    // -----------------------------------------------------------------------
    // GraphicsStateStack
    // -----------------------------------------------------------------------

    #[test]
    fn test_graphics_state_stack_push_pop() {
        let mut stack = GraphicsStateStack::default();
        stack.current_mut().line_width = 5.0;
        stack.push();
        // Inner state inherits outer
        assert!((stack.current().line_width - 5.0).abs() < 1e-6);
        // Modify inner
        stack.current_mut().line_width = 10.0;
        stack.pop();
        // Outer restored
        assert!((stack.current().line_width - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_graphics_state_stack_pop_at_bottom_no_panic() {
        let mut stack = GraphicsStateStack::default();
        // Popping beyond bottom should not panic (stack keeps at least one)
        stack.pop();
        stack.pop();
        // Should still have the default state
        let _ = stack.current();
    }

    #[test]
    fn test_graphics_state_stack_concat_ctm() {
        let mut stack = GraphicsStateStack::default();
        let scale = Matrix {
            a: 2.0,
            b: 0.0,
            c: 0.0,
            d: 2.0,
            e: 0.0,
            f: 0.0,
        };
        stack.concat_ctm(scale);
        let (x, y) = stack.current().ctm.transform_point(1.0, 1.0);
        assert!((x - 2.0).abs() < 1e-6);
        assert!((y - 2.0).abs() < 1e-6);
    }

    // -----------------------------------------------------------------------
    // CurrentPath
    // -----------------------------------------------------------------------

    #[test]
    fn test_current_path_starts_empty() {
        let p = CurrentPath::default();
        assert!(p.segments.is_empty());
    }

    #[test]
    fn test_current_path_move_to_updates_position() {
        let mut p = CurrentPath::default();
        p.move_to(10.0, 20.0);
        assert!((p.current_x - 10.0).abs() < 1e-6);
        assert!((p.current_y - 20.0).abs() < 1e-6);
        assert_eq!(p.segments.len(), 1);
    }

    #[test]
    fn test_current_path_line_to_updates_position() {
        let mut p = CurrentPath::default();
        p.move_to(0.0, 0.0);
        p.line_to(5.0, 7.0);
        assert!((p.current_x - 5.0).abs() < 1e-6);
        assert!((p.current_y - 7.0).abs() < 1e-6);
        assert_eq!(p.segments.len(), 2);
    }

    #[test]
    fn test_current_path_curve_to_updates_end_position() {
        let mut p = CurrentPath::default();
        p.move_to(0.0, 0.0);
        p.curve_to(1.0, 1.0, 2.0, 2.0, 3.0, 4.0);
        assert!((p.current_x - 3.0).abs() < 1e-6);
        assert!((p.current_y - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_current_path_close_appends_segment() {
        let mut p = CurrentPath::default();
        p.move_to(0.0, 0.0);
        p.close();
        assert_eq!(p.segments.len(), 2);
        assert!(matches!(p.segments.last(), Some(PathSegment::Close)));
    }

    #[test]
    fn test_current_path_rect_appends_segment() {
        let mut p = CurrentPath::default();
        p.rect(10.0, 20.0, 100.0, 50.0);
        assert_eq!(p.segments.len(), 1);
        assert!(matches!(
            p.segments[0],
            PathSegment::Rect(10.0, 20.0, 100.0, 50.0)
        ));
    }

    #[test]
    fn test_current_path_clear_resets() {
        let mut p = CurrentPath::default();
        p.move_to(1.0, 2.0);
        p.line_to(3.0, 4.0);
        p.clear();
        assert!(p.segments.is_empty());
    }

    // -----------------------------------------------------------------------
    // ClipState
    // -----------------------------------------------------------------------

    fn unit_clip_path(even_odd: bool) -> ClipPath {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.push_rect(tiny_skia::Rect::from_xywh(0.0, 0.0, 10.0, 10.0).expect("unit rect"));
        let path = pb.finish().expect("finished clip path");
        ClipPath { path, even_odd }
    }

    #[test]
    fn test_clip_state_default_is_unclipped() {
        let c = ClipState::default();
        assert!(c.is_unclipped(), "default clip state covers the whole page");
        assert!(c.paths().is_empty());
    }

    #[test]
    fn test_clip_state_intersect_adds_outline() {
        let mut c = ClipState::default();
        c.intersect(unit_clip_path(false));
        assert!(!c.is_unclipped());
        assert_eq!(c.paths().len(), 1);
        assert!(!c.paths()[0].even_odd, "W records nonzero winding rule");
    }

    #[test]
    fn test_clip_state_intersect_preserves_even_odd_rule() {
        let mut c = ClipState::default();
        c.intersect(unit_clip_path(true));
        assert!(c.paths()[0].even_odd, "W* records even-odd rule");
    }

    #[test]
    fn test_clip_state_clone_shares_region_identity() {
        let mut c = ClipState::default();
        c.intersect(unit_clip_path(false));
        let cloned = c.clone();
        assert!(
            c.same_region(&cloned),
            "a clone must share region identity (cheap O(1) q/Q save)"
        );
        assert!(
            !c.same_region(&ClipState::default()),
            "a distinct region must not compare equal"
        );
    }

    #[test]
    fn test_clip_state_intersect_is_copy_on_write() {
        // Intersecting through a clone must not disturb the original region —
        // q pushes a clone, a nested W extends only the clone, Q restores.
        let mut base = ClipState::default();
        base.intersect(unit_clip_path(false));
        let mut child = base.clone();
        child.intersect(unit_clip_path(false));
        assert_eq!(
            base.paths().len(),
            1,
            "original region is untouched by the child's intersect"
        );
        assert_eq!(child.paths().len(), 2);
        assert!(!base.same_region(&child));
    }
}
