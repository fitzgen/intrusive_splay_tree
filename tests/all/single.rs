use intrusive_splay_tree::{IntrusiveNode, Node, TreeOrd};
use std::cmp::Ordering;
use std::marker::PhantomData;

#[derive(Debug, Default)]
pub struct Single<'a> {
    pub value: usize,
    node: Node<'a>,
}

impl<'a> Single<'a> {
    pub fn new(x: usize) -> Single<'a> {
        Single {
            value: x,
            node: Default::default(),
        }
    }
}

pub struct SingleTree;

intrusive_splay_tree::impl_intrusive_node! {
    impl<'a> IntrusiveNode<'a> for SingleTree
    where
        type Elem = Single<'a>,
        node = node;
}

impl<'a> TreeOrd<'a, SingleTree> for Single<'a> {
    fn tree_cmp(&self, rhs: &Single<'a>) -> Ordering {
        self.value.cmp(&rhs.value)
    }
}

impl<'a> TreeOrd<'a, SingleTree> for usize {
    fn tree_cmp(&self, rhs: &Single<'a>) -> Ordering {
        self.cmp(&rhs.value)
    }
}
