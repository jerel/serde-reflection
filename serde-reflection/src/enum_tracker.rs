use std::{borrow::Borrow, collections::BTreeMap};

#[derive(Debug)]
pub(crate) struct EnumTracker {
    nodes: Vec<Node>,
    breadcrumbs: Vec<usize>,
}

impl EnumTracker {
    pub(crate) fn new() -> Self {
        Self {
            nodes: vec![],
            breadcrumbs: vec![],
        }
    }

    fn track<F, O>(&mut self, name: String, max_index: usize, f: F) -> O
    where
        F: FnOnce() -> O,
    {
        self.open_node(name, max_index);
        let output = f();
        self.close_node();
        output
    }

    pub(crate) fn open_node(&mut self, name: String, max_index: usize) -> &mut Self {
        let index = if !self.node_exists(&name) {
            let mut node = Node::new(name.clone(), max_index);
            let index = self.nodes.len();
            node.this = index;
            self.nodes.push(node);

            // no need to record ourselves as a child if we are the root node
            if index > 0 {
                let parent = self.get_active_node(None);
                parent.children.insert(parent.index, index);
            }
            index
        } else {
            let node = self.get_active_node(Some(name.clone()));
            node.this
        };

        self.breadcrumbs.push(index);
        println!("open_node: {:?} {:#?}", name, self);
        self
    }

    fn node_exists(&mut self, name: &String) -> bool {
        // this needs to be based on a position not `skip` (count)
        let nearest_ancestor = *self.breadcrumbs.last().unwrap_or(&0);
        self.nodes
            .iter()
            // .skip(nearest_ancestor)
            .any(|n| n.name == *name)
    }

    fn get_active_node(&mut self, name: Option<String>) -> &mut Node {
        if let Some(name) = name {
            let index = self.nodes.iter().position(|n| n.name == name).unwrap();

            self.nodes
                .get_mut(index)
                .expect("The node for a name should always exist.")
        } else {
            self.nodes
                .get_mut(*self.breadcrumbs.last().unwrap())
                .expect("The node for a breadcrumb should always exist.")
        }
    }

    fn get_child_node(self, node: Node) -> Option<Node> {
        if let Some(index) = node.children.get(&node.index) {
            if let Some(child) = self.nodes.get(*index as usize) {
                return Some(child.clone());
            }
        }

        None
    }

    pub(crate) fn next_incomplete_variant(&mut self) -> usize {
        self.nodes
            .get(*self.breadcrumbs.last().unwrap())
            .unwrap()
            .index
    }

    fn advance_variant(&mut self) {
        let active = self.nodes.get(*self.breadcrumbs.last().unwrap()).unwrap();

        if active.complete(&self.nodes) {
            ()
        } else if active.state == NodeState::Discovery || active.children.is_empty() {
            let active = self.get_active_node(None);
            active.advance_index();
        } else {
            if let Some(index) = active.children.get(&active.index) {
                let child = self.nodes.get(*index as usize).unwrap();
                println!("{:#?} complete: {}", child, child.complete(&self.nodes));

                if child.complete(&self.nodes) {
                    let active = self.get_active_node(None);
                    active.advance_index();
                }
            } else {
                // we're in the Completion state and no child was found for this variant, move on
                let active = self.get_active_node(None);
                active.advance_index();
            }
        };
        println!("advance_variant {:#?}", self);
    }

    // fn get_active_node(&mut self) -> &mut Node {
    //     let mut cursor = 0;

    //     loop {
    //         let node = self
    //             .nodes
    //             .get(cursor)
    //             .expect("The node for a cursor should always exist.");
    //         match node.cursor {
    //             Some(c) => cursor = c,
    //             None => {
    //                 return self
    //                     .nodes
    //                     .get_mut(cursor)
    //                     .expect("The node for a cursor should always exist.")
    //             }
    //         }
    //     }
    // }

    pub(crate) fn close_node(&mut self) -> &mut Self {
        // let index = *self.breadcrumbs.pop().unwrap();
        // let mut node = self.nodes.get_mut(index).unwrap();
        // node.cursor = None;
        // self.scope_cursor = self.scope_cursor - 1;
        self.advance_variant();
        self.breadcrumbs.pop();
        self
    }

    pub(crate) fn all_complete(&mut self) -> bool {
        // no enums were found
        self.nodes.len() == 0
            || self
                .nodes
                .get(0)
                .expect("One node should always exist")
                .complete(&self.nodes)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum NodeState {
    Discovery,
    Completion,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Node {
    name: String,
    this: usize,
    index: usize,
    max_index: usize,
    children: BTreeMap<usize, usize>,
    state: NodeState,
}

impl Node {
    fn new(name: String, max_index: usize) -> Self {
        Self {
            name,
            this: 0,
            index: 0,
            max_index,
            children: BTreeMap::new(),
            state: NodeState::Discovery,
        }
    }

    fn advance_index(&mut self) -> &Self {
        if self.index == self.max_index && self.state == NodeState::Discovery {
            self.state = NodeState::Completion;
            self.index = 0;
        } else if self.index < self.max_index {
            self.index = self.index + 1;
        }

        self
    }

    fn complete(&self, nodes: &Vec<Node>) -> bool {
        self.index == self.max_index
            && self.state == NodeState::Completion
            && self
                .children
                .iter()
                .map(|(_variant_index, index)| nodes.get(*index as usize).unwrap())
                .all(|child| child.complete(nodes))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn basic_tree() -> EnumTracker {
        EnumTracker {
            nodes: vec![
                Node {
                    name: "enum1".to_string(),
                    this: 0,
                    index: 1,
                    max_index: 1,
                    children: BTreeMap::from([(0, 1), (1, 2)]),
                    state: NodeState::Completion,
                },
                Node {
                    name: "enum1child1".to_string(),
                    this: 1,
                    index: 0,
                    max_index: 0,
                    children: BTreeMap::new(),
                    state: NodeState::Completion,
                },
                Node {
                    name: "enum1child2".to_string(),
                    this: 2,
                    index: 0,
                    max_index: 0,
                    children: BTreeMap::from([(0, 3)]),
                    state: NodeState::Completion,
                },
                Node {
                    name: "enum1child2child1".to_string(),
                    this: 3,
                    index: 0,
                    max_index: 0,
                    children: BTreeMap::new(),
                    state: NodeState::Completion,
                },
            ],
            breadcrumbs: vec![],
        }
    }

    #[test]
    fn test_enum_tracker_can_add_new_enum_with_children() {
        let mut tracker = EnumTracker::new();
        let trace = |tracker: &mut EnumTracker| {
            // first iteration
            tracker.open_node("enum1".to_string(), 1);
            assert_eq!(tracker.breadcrumbs.last().unwrap(), &0);
            tracker.open_node("enum1child1".to_string(), 0);
            assert_eq!(tracker.breadcrumbs.last().unwrap(), &1);
            tracker.close_node();
            tracker.close_node();

            // second iteration
            tracker.open_node("enum1".to_string(), 1);
            assert_eq!(tracker.next_incomplete_variant(), 1);
            tracker.open_node("enum1child2".to_string(), 0);
            assert_eq!(tracker.breadcrumbs.last().unwrap(), &2);
            tracker.open_node("enum1child2child1".to_string(), 0);
            assert_eq!(tracker.breadcrumbs.last().unwrap(), &3);
            tracker.close_node();
            tracker.close_node();
            tracker.close_node();

            // third iteration
            tracker.open_node("enum1".to_string(), 1);
            tracker.close_node();
        };

        // at the beginning we should have only the root node
        assert!(tracker.nodes.is_empty());
        trace(&mut tracker);

        assert_eq!(tracker.nodes, basic_tree().nodes, "{:#?}", tracker);
        // and our scope tracker is back at the beginning where we started
        assert!(tracker.breadcrumbs.is_empty());

        trace(&mut tracker);

        // if this passes then we know that simply walking the tree again didn't add duplicate nodes
        assert_eq!(tracker.nodes, basic_tree().nodes);
    }

    #[test]
    fn test_enum_tracker_can_iterate_variants() {
        let mut tracker = EnumTracker::new();
        // add a node and one child on the first pass
        tracker.open_node("enum1".to_string(), 1);
        assert_eq!(tracker.next_incomplete_variant(), 0);
        tracker.open_node("enum1child1".to_string(), 0);
        assert_eq!(tracker.next_incomplete_variant(), 0);
        tracker.close_node();
        tracker.close_node();

        // second pass which adds another child
        tracker.open_node("enum1".to_string(), 1);
        assert_eq!(tracker.next_incomplete_variant(), 1);
        tracker.open_node("enum1child2".to_string(), 0);
        assert_eq!(tracker.next_incomplete_variant(), 0);
        tracker.close_node();
        tracker.close_node();

        assert!(!tracker.all_complete(), "{:#?}", tracker);

        // third pass which finishes up
        tracker.open_node("enum1".to_string(), 1);
        assert_eq!(tracker.next_incomplete_variant(), 0);
        tracker.open_node("enum1child2".to_string(), 0);
        assert_eq!(tracker.next_incomplete_variant(), 0);
        tracker.close_node();
        tracker.close_node();

        assert!(tracker.all_complete(), "{:#?}", tracker);
    }

    #[test]
    fn test_enum_tracker_is_complete_when_no_enums_found() {
        let mut tracker = EnumTracker::new();
        assert!(tracker.all_complete());
    }
}
