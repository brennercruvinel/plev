//! the row of a list's item, by sense (the checkpoint b of task-69: the
//! rules' row showed "3.02 5.02 2" with no name and let "reject cut" fall
//! alone to a line of its own). what the item is (its picture, its texts,
//! its decisions as a badge), its numbers (a small stat each, named by the
//! state's key, as the counts are), its actions (checkboxes, buttons):
//! each sense of two nodes or more is a row of its own, one under the
//! other, each as wide as the item and wrapping inside itself when it
//! must; a row of one node per sense (a text and a checkbox) stays one
//! flat row.

use std::collections::BTreeSet;

use plevir::{
    Align, Axis, Component, ControlSize, IrNode, Props, Repeat, Size, SpacingToken, TypographyRole,
};

use crate::ids::{Anchor, anchors};
use crate::list::List;
use crate::rules::{Build, stack};

/// a state's key as a label: `cut_secs` reads "cut secs".
fn label(key: &accord::StateKey) -> String {
    key.as_str().replace('_', " ")
}

impl Build<'_> {
    /// what an item is: an image per frame, a text per text, a badge per
    /// decision.
    fn about(&mut self, l: &List) -> Vec<IrNode> {
        let mut out = Vec::new();
        for at in &l.frames {
            let mut image = IrNode::new(l.item.clone());
            image.component = Some(Component::Image);
            image.props.control_size = Some(ControlSize::Xl);
            image.bind.state.insert("source".to_owned(), at.clone());
            let name = format!("{}.{}", l.item, at.key);
            out.push(self.bound(image, &name, at.key.as_str().to_owned()));
        }
        for at in &l.texts {
            let mut text = IrNode::new(l.item.clone());
            text.component = Some(Component::Text);
            // the text takes the room its row has, so it wraps only when
            // the row cannot hold it
            text.size = Size {
                flex: Some(1),
                ..Size::default()
            };
            text.bind.state.insert("text".to_owned(), at.clone());
            let name = format!("{}.{}", l.item, at.key);
            text.sem = self
                .namer
                .bound(&Anchor::State(at.clone()), &name, String::new())
                .0;
            out.push(text);
        }
        for at in &l.states {
            let mut badge = IrNode::new(l.item.clone());
            badge.component = Some(Component::Badge);
            badge.bind.state.insert("text".to_owned(), at.clone());
            let name = format!("{}.{}", l.item, at.key);
            badge.sem = self
                .namer
                .bound(&Anchor::State(at.clone()), &name, String::new())
                .0;
            out.push(badge);
        }
        out
    }

    /// an item's numbers: a stat each, the inline readout, named by its
    /// key.
    fn numbers(&mut self, l: &List) -> Vec<IrNode> {
        let mut out = Vec::new();
        for at in &l.numbers {
            let mut stat = IrNode::new(l.item.clone());
            stat.component = Some(Component::Stat);
            stat.props = Props {
                typography: Some(TypographyRole::BaseM),
                ..Props::default()
            };
            stat.bind.state.insert("value".to_owned(), at.clone());
            let name = format!("{}.{}", l.item, at.key);
            out.push(self.bound(stat, &name, label(&at.key)));
        }
        out
    }

    /// an item's actions: a checkbox per bool it flips, a button per
    /// other action.
    fn actions(&mut self, l: &List) -> Vec<IrNode> {
        let mut out = Vec::new();
        for (at, flip) in &l.checks {
            let mut check = IrNode::new(l.item.clone());
            check.component = Some(Component::Checkbox);
            check.bind.state.insert("checked".to_owned(), at.clone());
            check.bind.actions.insert("toggle".to_owned(), flip.clone());
            let name = format!("{}.{}", l.item, at.key);
            out.push(self.bound(check, &name, at.key.as_str().to_owned()));
        }
        for (cap, text) in &l.buttons {
            let mut button = IrNode::new(l.item.clone());
            button.component = Some(Component::Button);
            // an item's own buttons are small, as a row of items draws
            // them, so its actions keep to one line where the width holds
            button.props.control_size = Some(ControlSize::Xs);
            button
                .bind
                .actions
                .insert("activate".to_owned(), cap.clone());
            let name = format!("{}.{}", l.item, cap.action);
            out.push(self.bound(button, &name, text.clone()));
        }
        out
    }

    /// `nodes` of one sense: the node alone, or a row of them, its id in
    /// the template (`<item>.<sense>`).
    fn sense(&mut self, l: &List, sense: &str, mut nodes: Vec<IrNode>) -> Option<IrNode> {
        if nodes.len() < 2 {
            return nodes.pop();
        }
        let mut held = BTreeSet::new();
        for n in &nodes {
            held.extend(anchors(n));
        }
        let id = self
            .namer
            .in_template(&held, &format!("{}.{sense}", l.item));
        let mut group = IrNode::new(id);
        group.layout = stack(Axis::Horizontal, SpacingToken::Sm, Some(Align::Center));
        group.children = nodes;
        Some(group)
    }

    /// a list over `l.each`: one row per item, its template bound to the
    /// item by `[{item}]`; the template root is named by its item.
    pub(crate) fn list(&mut self, l: &List) -> IrNode {
        let one_each = l.frames.len() + l.texts.len() + l.states.len() <= 1
            && l.numbers.len() <= 1
            && l.checks.len() + l.buttons.len() <= 1;
        let about = self.about(l);
        let numbers = self.numbers(l);
        let actions = self.actions(l);
        let row: Vec<IrNode> = if one_each {
            about.into_iter().chain(numbers).chain(actions).collect()
        } else {
            [("about", about), ("numbers", numbers), ("actions", actions)]
                .into_iter()
                .filter_map(|(sense, nodes)| self.sense(l, sense, nodes))
                .collect()
        };
        let mut held = BTreeSet::new();
        for n in &row {
            held.extend(anchors(n));
        }
        let mut item = IrNode::new(self.namer.template(&l.each, &held, l.item.as_str()));
        // one flat row; or the senses one under the other, each as wide as
        // the item, wrapping inside itself when it must, and the items a
        // step apart
        let (row_layout, apart) = if one_each {
            let row = stack(Axis::Horizontal, SpacingToken::Sm, Some(Align::Center));
            (row, SpacingToken::Xs)
        } else {
            (
                stack(Axis::Vertical, SpacingToken::Xs, None),
                SpacingToken::Md,
            )
        };
        item.layout = row_layout;
        item.children = row;
        let mut list = IrNode::new(l.each.node.clone());
        list.component = Some(Component::List);
        list.layout = stack(Axis::Vertical, apart, None);
        list.repeat = Some(Box::new(Repeat {
            each: l.each.clone(),
            item,
        }));
        let name = format!("{}.{}", l.each.node, l.each.key);
        self.bound(list, &name, l.each.key.as_str().to_owned())
    }
}
