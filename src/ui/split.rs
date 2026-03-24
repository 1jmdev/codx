use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::ui::Pane;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub enum WindowNode {
    Leaf(Pane),
    Split {
        direction: SplitDirection,
        ratio: u16,
        first: Box<WindowNode>,
        second: Box<WindowNode>,
    },
}

impl WindowNode {
    pub fn pane(&self, pane_id: u64) -> Option<&Pane> {
        match self {
            WindowNode::Leaf(pane) => (pane.id() == pane_id).then_some(pane),
            WindowNode::Split { first, second, .. } => {
                first.pane(pane_id).or_else(|| second.pane(pane_id))
            }
        }
    }

    pub fn pane_mut(&mut self, pane_id: u64) -> Option<&mut Pane> {
        match self {
            WindowNode::Leaf(pane) => (pane.id() == pane_id).then_some(pane),
            WindowNode::Split { first, second, .. } => {
                if let Some(pane) = first.pane_mut(pane_id) {
                    Some(pane)
                } else {
                    second.pane_mut(pane_id)
                }
            }
        }
    }

    pub fn collect_leaf_ids(&self, ids: &mut Vec<u64>) {
        match self {
            WindowNode::Leaf(pane) => ids.push(pane.id()),
            WindowNode::Split { first, second, .. } => {
                first.collect_leaf_ids(ids);
                second.collect_leaf_ids(ids);
            }
        }
    }

    pub fn collect_layout<'a>(&'a self, area: Rect, output: &mut Vec<(u64, Rect, &'a Pane)>) {
        match self {
            WindowNode::Leaf(pane) => output.push((pane.id(), area, pane)),
            WindowNode::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let constraints = [
                    Constraint::Percentage(*ratio),
                    Constraint::Percentage(100u16.saturating_sub(*ratio)),
                ];
                let areas = match direction {
                    SplitDirection::Horizontal => Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(constraints)
                        .split(area),
                    SplitDirection::Vertical => Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(constraints)
                        .split(area),
                };
                first.collect_layout(areas[0], output);
                second.collect_layout(areas[1], output);
            }
        }
    }

    pub fn split_leaf(
        &mut self,
        target_pane_id: u64,
        new_pane: Pane,
        direction: SplitDirection,
    ) -> bool {
        match self {
            WindowNode::Leaf(pane) if pane.id() == target_pane_id => {
                let current = std::mem::replace(pane, Pane::new(u64::MAX, u64::MAX));
                *self = WindowNode::Split {
                    direction,
                    ratio: 50,
                    first: Box::new(WindowNode::Leaf(current)),
                    second: Box::new(WindowNode::Leaf(new_pane)),
                };
                true
            }
            WindowNode::Leaf(_) => false,
            WindowNode::Split { first, second, .. } => {
                if first.pane(target_pane_id).is_some() {
                    first.split_leaf(target_pane_id, new_pane, direction)
                } else {
                    second.split_leaf(target_pane_id, new_pane, direction)
                }
            }
        }
    }

    pub fn resize_split(&mut self, target_pane_id: u64, delta: i16) -> bool {
        match self {
            WindowNode::Leaf(_) => false,
            WindowNode::Split {
                ratio,
                first,
                second,
                ..
            } => {
                let first_contains = first.pane(target_pane_id).is_some();
                let second_contains = second.pane(target_pane_id).is_some();
                if first_contains || second_contains {
                    let next = if first_contains {
                        *ratio as i16 + delta
                    } else {
                        *ratio as i16 - delta
                    };
                    *ratio = next.clamp(20, 80) as u16;
                    true
                } else {
                    first.resize_split(target_pane_id, delta)
                        || second.resize_split(target_pane_id, delta)
                }
            }
        }
    }
}
