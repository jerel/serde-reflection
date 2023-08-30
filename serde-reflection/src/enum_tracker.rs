use std::collections::BTreeMap;

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

    pub(crate) fn open_node(&mut self, name: String, max_index: usize) {
        let index = if self.node_exists(&name) {
            let node = self.get_active_node(Some(name.clone()));
            node.this
        } else {
            let mut node = Node::new(name.clone(), max_index);
            let index = self.nodes.len();
            node.this = index;
            self.nodes.push(node);

            // no need to record ourselves as a child if we are the root node
            if self.nodes.len() > 1 {
                let parent = self.get_active_node(None);
                parent.children.insert(parent.index, index);
            }
            index
        };

        println!("open_node: {:?} {:#?}", name, self);
        // prevent entering into a recursive variant a second time
        if self.breadcrumbs.contains(&index) {
            // record the recursion
            let parent = self.nodes.get_mut(index).unwrap();
            // since we're recursing advance_variant won't be called so we have to manually
            // advance the index past the one which points to Self; and if this is the last
            // variant then `parent` will get set to Completed when we advance
            if !parent.recursive_variants.contains(&parent.index) {
            parent.advance_index(true);
            parent.children.insert(parent.index, parent.this);
            parent.recursive_variants.push(parent.index);
            }
        }

        self.breadcrumbs.push(index);
    }

    fn node_exists(&mut self, name: &String) -> bool {
        // this needs to be based on a position not `skip` (count)
        // let nearest_ancestor = *self.breadcrumbs.last().unwrap_or(&0);
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

    pub(crate) fn next_variant_index(&mut self) -> usize {
        self.nodes
            .get(*self.breadcrumbs.last().unwrap())
            .unwrap()
            .index
    }

    fn advance_variant(&mut self) {
        let active = if let Some(index) = self.breadcrumbs.last() {
            self.nodes.get(*index).unwrap()
        } else {
            unreachable!("open_node and close_node usage isn't paired or (this is a bug) recursion was improperly handled");
        };

        if active.complete(&self.nodes)
            || active.state == NodeState::Discovery
            || active.children.is_empty()
        {
            let active = self.get_active_node(None);
            active.advance_index(false);
        } else {
            if let Some(index) = active.children.get(&active.index) {
                let child = self.nodes.get(*index as usize).unwrap();

                let variant_complete = child.complete(&self.nodes);
                if variant_complete {
                    let active = self.get_active_node(None);
                    active.advance_index(variant_complete);
                }
            } else {
                // we're in the Completion state and no child was found for this variant, move on
                let active = self.get_active_node(None);
                active.advance_index(false);
            }

            // self.advance_variant();
        };
        println!("advance_variant {:#?}", self);
    }

    pub(crate) fn close_node(&mut self) -> &mut Self {
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
    Completed,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Node {
    name: String,
    this: usize,
    index: usize,
    max_index: usize,
    children: BTreeMap<usize, usize>,
    state: NodeState,
    recursive_variants: Vec<usize>,
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
            recursive_variants: vec![],
        }
    }

    fn advance_index(&mut self, variant_complete: bool) -> &Self {
        if self.index == self.max_index {
            if self.state == NodeState::Discovery {
                // if we've completed the disovery phase then we start over for one more pass
                self.state = NodeState::Completion;
                self.index = 0;
            } else if self.state == NodeState::Completion || variant_complete {
                // advance_index is called during close_node so we can safely toggle the state if we've made it to the final index
                self.state = NodeState::Completed;
            }
        } else if self.index < self.max_index
            && [NodeState::Discovery, NodeState::Completion].contains(&self.state)
        {
            self.index = self.index + 1;
        }

        self
    }

    fn complete(&self, nodes: &Vec<Node>) -> bool {
        self.state == NodeState::Completed
            || (self.index == self.max_index
                && self.state == NodeState::Completion
                && self
                    .children
                    .iter()
                    .map(|(_variant_index, index)| nodes.get(*index as usize).unwrap())
                    .all(|child| child.complete(nodes)))
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
                    state: NodeState::Completed,
                    recursive_variants: vec![],
                },
                Node {
                    name: "enum1child1".to_string(),
                    this: 1,
                    index: 0,
                    max_index: 0,
                    children: BTreeMap::new(),
                    state: NodeState::Completed,
                    recursive_variants: vec![],
                },
                Node {
                    name: "enum1child2".to_string(),
                    this: 2,
                    index: 0,
                    max_index: 0,
                    children: BTreeMap::from([(0, 3)]),
                    state: NodeState::Completed,
                    recursive_variants: vec![],
                },
                Node {
                    name: "enum1child2child1".to_string(),
                    this: 3,
                    index: 0,
                    max_index: 0,
                    children: BTreeMap::new(),
                    state: NodeState::Completed,
                    recursive_variants: vec![],
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
            // assert_eq!(tracker.next_variant_index(), 0, "{:#?}", tracker);
            tracker.open_node("enum1child1".to_string(), 0);
            assert_eq!(tracker.breadcrumbs.last().unwrap(), &1);
            assert_eq!(tracker.next_variant_index(), 0);
            tracker.close_node();
            tracker.close_node();

            println!("one: {:#?}", tracker);

            // second iteration which finishes Discovery
            tracker.open_node("enum1".to_string(), 1);
            assert_eq!(tracker.next_variant_index(), 1);
            tracker.open_node("enum1child2".to_string(), 0);
            assert_eq!(tracker.breadcrumbs.last().unwrap(), &2);
            assert_eq!(tracker.next_variant_index(), 0);
            tracker.open_node("enum1child2child1".to_string(), 0);
            assert_eq!(tracker.breadcrumbs.last().unwrap(), &3);
            tracker.close_node();
            tracker.close_node();
            tracker.close_node();

            println!("two: {:#?}", tracker);
            // everything is in Completion now
            tracker.open_node("enum1".to_string(), 1);
            // assert_eq!(tracker.next_variant_index(), 0, "{:#?}", tracker);
            tracker.open_node("enum1child1".to_string(), 0);
            assert_eq!(tracker.next_variant_index(), 0);
            tracker.close_node();
            tracker.close_node();

            tracker.open_node("enum1".to_string(), 1);
            assert_eq!(tracker.next_variant_index(), 1, "{:#?}", tracker);
            tracker.open_node("enum1child2".to_string(), 0);
            assert_eq!(tracker.next_variant_index(), 0);
            tracker.open_node("enum1child2child1".to_string(), 0);
            assert_eq!(tracker.breadcrumbs.last().unwrap(), &3);
            tracker.close_node();
            tracker.close_node();
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
        assert_eq!(tracker.next_variant_index(), 0);
        tracker.open_node("enum1child1".to_string(), 0);
        assert_eq!(tracker.next_variant_index(), 0);
        tracker.close_node();
        tracker.close_node();

        // second pass which adds another child
        tracker.open_node("enum1".to_string(), 1);
        assert_eq!(tracker.next_variant_index(), 1);
        tracker.open_node("enum1child2".to_string(), 0);
        assert_eq!(tracker.next_variant_index(), 0);
        tracker.close_node();
        tracker.close_node();

        assert!(!tracker.all_complete(), "{:#?}", tracker);

        // third pass which finishes up
        tracker.open_node("enum1".to_string(), 1);
        assert_eq!(tracker.next_variant_index(), 0);
        tracker.open_node("enum1child2".to_string(), 0);
        assert_eq!(tracker.next_variant_index(), 0);
        tracker.close_node();
        tracker.close_node();

        assert!(tracker.all_complete(), "{:#?}", tracker);
    }

    #[test]
    fn test_enum_tracker_is_complete_when_no_enums_found() {
        let mut tracker = EnumTracker::new();
        assert!(tracker.all_complete());
    }

    #[test]
    fn test_enum_tracker_handles_recursion() {
        let mut tracker = EnumTracker::new();
        tracker.open_node("enum1".to_string(), 1);
        assert_eq!(tracker.next_variant_index(), 0, "{:#?}", tracker);
        tracker.open_node("enum1child1".to_string(), 0);
        assert_eq!(tracker.next_variant_index(), 0, "{:#?}", tracker);
        tracker.close_node();
        tracker.close_node();

        // second iteration from the top, we mimick the second variant of enum1 being Box<Self>
        tracker.open_node("enum1".to_string(), 1);
        assert_eq!(tracker.next_variant_index(), 1, "{:#?}", tracker);
        assert_eq!(
            tracker
                .nodes
                .iter()
                .map(|n| &n.state)
                .collect::<Vec<&NodeState>>(),
            vec![&NodeState::Discovery, &NodeState::Completion],
            "{:#?}",
            tracker
        );
        // we're now in a recursive loop looking at Self
        tracker.open_node("enum1".to_string(), 1);
        assert_eq!(tracker.next_variant_index(), 0, "{:#?}", tracker);
        assert_eq!(
            tracker
                .nodes
                .iter()
                .map(|n| &n.state)
                .collect::<Vec<&NodeState>>(),
            vec![&NodeState::Completion, &NodeState::Completion],
            "{:#?}",
            tracker
        );
        // we get sent to look at enum1child1 in the Completion scan
        tracker.open_node("enum1child1".to_string(), 0);
        assert_eq!(tracker.next_variant_index(), 0, "{:#?}", tracker);
        tracker.close_node();
        assert_eq!(
            tracker
                .nodes
                .iter()
                .map(|n| &n.state)
                .collect::<Vec<&NodeState>>(),
            vec![&NodeState::Completion, &NodeState::Completed],
            "{:#?}",
            tracker
        );
        // we exit the loop because we looked at a variant other than Self
        tracker.close_node();
        panic!("{:#?}", tracker);

        // third iteration from the top
        // tracker.open_node("enum1".to_string(), 1);
        // assert_eq!(tracker.next_variant_index(), 1, "{:#?}", tracker);
        // assert_eq!(
        //     tracker
        //         .nodes
        //         .iter()
        //         .map(|n| &n.state)
        //         .collect::<Vec<&NodeState>>(),
        //     vec![&NodeState::Completion, &NodeState::Completed],
        //     "{:#?}",
        //     tracker
        // );
        // tracker.open_node("enum1".to_string(), 1);
        // assert_eq!(tracker.next_variant_index(), 0, "{:#?}", tracker);
        // tracker.close_node();
        // tracker.close_node();


        tracker.close_node();
    }
}
