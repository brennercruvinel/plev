//! finding a node of a surface's tree by its id: the path to it from the
//! root, the node at a path, every id under a node. a template (a
//! repeat's item) is one node here, the one the IR holds; its instances
//! exist only where a surface unrolls it.

use accord::SemId;

use crate::ir::IrNode;
use crate::item::{self, Pattern};

/// one step down from a node: to its child at an index, or to its
/// repeat's item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Step {
    Child(usize),
    Item,
}

/// the steps from `root` to the node `id`, `root` itself the empty path.
pub(super) fn path(root: &IrNode, id: &SemId) -> Option<Vec<Step>> {
    // depth first, each node with the path to it; the verifier bounds
    // the depth, and the stack keeps a deep tree off the call stack
    let mut stack = vec![(root, Vec::new())];
    while let Some((node, at)) = stack.pop() {
        if node.sem == *id {
            return Some(at);
        }
        if let Some(repeat) = &node.repeat {
            let mut next = at.clone();
            next.push(Step::Item);
            stack.push((&repeat.item, next));
        }
        for (i, child) in node.children.iter().enumerate().rev() {
            let mut next = at.clone();
            next.push(Step::Child(i));
            stack.push((child, next));
        }
    }
    None
}

/// the node at `path` under `root`.
pub(super) fn at<'a>(root: &'a IrNode, path: &[Step]) -> Option<&'a IrNode> {
    let mut node = root;
    for step in path {
        node = match step {
            Step::Child(i) => node.children.get(*i)?,
            Step::Item => &node.repeat.as_ref()?.item,
        };
    }
    Some(node)
}

/// the node at `path` under `root`, to change it.
pub(super) fn at_mut<'a>(root: &'a mut IrNode, path: &[Step]) -> Option<&'a mut IrNode> {
    let mut node = root;
    for step in path {
        node = match step {
            Step::Child(i) => node.children.get_mut(*i)?,
            Step::Item => &mut node.repeat.as_mut()?.item,
        };
    }
    Some(node)
}

/// the node `id` under `root`.
pub(super) fn find<'a>(root: &'a IrNode, id: &SemId) -> Option<&'a IrNode> {
    path(root, id).and_then(|p| at(root, &p))
}

/// every node under `n`, `n` included, in tree order: children, then the
/// repeat's item.
pub(super) fn nodes(n: &IrNode) -> Vec<&IrNode> {
    let mut out = Vec::new();
    let mut stack = vec![n];
    while let Some(node) = stack.pop() {
        out.push(node);
        if let Some(repeat) = &node.repeat {
            stack.push(&repeat.item);
        }
        stack.extend(node.children.iter().rev());
    }
    out
}

/// every id under `n`, `n` included, in tree order.
pub(super) fn ids(n: &IrNode) -> Vec<SemId> {
    nodes(n).into_iter().map(|node| node.sem.clone()).collect()
}

/// the template under `root` that makes `id` for one item, when `id` is
/// an instance: `review.cut[{item}]` for `review.cut[3]`.
pub(super) fn template_of(root: &IrNode, id: &SemId) -> Option<SemId> {
    nodes(root)
        .into_iter()
        .filter(|node| item::slots(node.sem.as_str()) == 1)
        .find(|node| Pattern::of(node.sem.as_str()).is_some_and(|p| p.matches(id.as_str())))
        .map(|node| node.sem.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Repeat;

    fn id(text: &str) -> SemId {
        SemId::parse(text).unwrap()
    }

    fn tree() -> IrNode {
        let mut root = IrNode::new(id("r"));
        let mut list = IrNode::new(id("r.list"));
        let mut item = IrNode::new(id("r.item[{item}]"));
        item.children.push(IrNode::new(id("r.item[{item}].go")));
        list.repeat = Some(Box::new(Repeat {
            each: "app:items".parse().unwrap(),
            item,
        }));
        root.children.push(IrNode::new(id("r.title")));
        root.children.push(list);
        root
    }

    #[test]
    fn a_path_reaches_a_child_and_a_template() {
        let root = tree();
        assert_eq!(path(&root, &id("r")), Some(vec![]));
        assert_eq!(path(&root, &id("r.title")), Some(vec![Step::Child(0)]));
        let go = path(&root, &id("r.item[{item}].go")).unwrap();
        assert_eq!(go, [Step::Child(1), Step::Item, Step::Child(0)]);
        assert_eq!(
            at(&root, &go).map(|n| n.sem.as_str()),
            Some("r.item[{item}].go")
        );
        assert_eq!(path(&root, &id("r.item[3].go")), None);
        let names: Vec<String> = ids(&root).iter().map(ToString::to_string).collect();
        assert_eq!(
            names,
            [
                "r",
                "r.title",
                "r.list",
                "r.item[{item}]",
                "r.item[{item}].go"
            ]
        );
    }

    #[test]
    fn an_instance_names_its_template() {
        let root = tree();
        assert_eq!(
            template_of(&root, &id("r.item[3].go")),
            Some(id("r.item[{item}].go"))
        );
        assert_eq!(
            template_of(&root, &id("r.item[3]")),
            Some(id("r.item[{item}]"))
        );
        assert_eq!(template_of(&root, &id("r.title")), None);
    }
}
