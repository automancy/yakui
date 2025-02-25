use crate::geometry::Rect;

/// A wrapper around [`Rect`][crate::geometry::Rect] to represent different clipping rect states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClipRect {
    /// Dummy value. The clipping rect will be resolved right after layouting is done. You should not see this.
    Unresolved,
    /// The clipping rect has been overridden to be the entire viewport, effectively escaping out of the parent's clipping rect.
    EntireViewport,
    /// The clipping rect has already been resolved, it has a known value.
    Resolved(Rect),
}

impl ClipRect {
    /// Gets a sensible [`Rect`][crate::geometry::Rect] from the state of clipping rect resolution.
    pub fn get_rect(self, viewport: Rect) -> Rect {
        match self {
            ClipRect::Unresolved => Rect::ZERO,
            ClipRect::EntireViewport => viewport,
            ClipRect::Resolved(rect) => rect,
        }
    }

    /// Attempts to resolve the clipping rect.
    pub fn resolve_rect(self, layout_rect: Rect, parent_rect: Rect, viewport: Rect) -> Rect {
        match self {
            ClipRect::Unresolved => parent_rect.constrain(layout_rect),
            ClipRect::EntireViewport => viewport,
            ClipRect::Resolved(rect) => rect,
        }
    }
}
