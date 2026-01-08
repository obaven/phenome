//! Geometry primitives for UI-core layouts.

/// 2D point in UI coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiPoint {
    pub x: u16,
    pub y: u16,
}

impl UiPoint {
    /// Construct a point.
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

/// Rectangle in UI coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl UiRect {
    /// Construct a rectangle.
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns true if the point lies inside the rectangle.
    pub fn contains(&self, point: UiPoint) -> bool {
        point.x >= self.x
            && point.x < self.x.saturating_add(self.width)
            && point.y >= self.y
            && point.y < self.y.saturating_add(self.height)
    }

    /// Shrink the rectangle by the given margin.
    pub fn inner(&self, margin: UiMargin) -> Self {
        let x = self.x.saturating_add(margin.horizontal);
        let y = self.y.saturating_add(margin.vertical);
        let width = self.width.saturating_sub(margin.horizontal.saturating_mul(2));
        let height = self.height.saturating_sub(margin.vertical.saturating_mul(2));
        Self { x, y, width, height }
    }
}

/// Horizontal/vertical margins for UI geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiMargin {
    pub horizontal: u16,
    pub vertical: u16,
}
