use thunderdome::Index;

use crate::layout::LayoutDom;
use crate::paint::PaintDom;
use crate::{Constraints, Widget};

use super::Dom;

#[derive(Debug)]
pub struct RootWidget {
    index: Index,
}

impl Widget for RootWidget {
    type Props = ();
    type Response = ();

    fn new(index: Index, _props: Self::Props) -> Self {
        Self { index }
    }

    fn update(&mut self, _props: Self::Props) {}
    fn respond(&mut self) -> Self::Response {}

    fn layout(&self, dom: &Dom, layout: &mut LayoutDom, constraints: Constraints) -> glam::Vec2 {
        let node = dom.get(self.index).unwrap();

        for &child in &node.children {
            layout.calculate(dom, child, constraints);
        }

        constraints.max
    }

    fn paint(&self, dom: &Dom, layout: &LayoutDom, paint: &mut PaintDom) {
        let node = dom.get(self.index).unwrap();

        for &child in &node.children {
            let child = dom.get(child).unwrap();
            child.widget.paint(dom, layout, paint);
        }
    }
}