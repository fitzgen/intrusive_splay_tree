use intrusive_splay_tree::{impl_intrusive_node, Node, SplayTree, TreeOrd};
use std::cmp::{min, Ordering};

#[derive(Debug, Default)]
pub struct Multiple<'a> {
    by_x: Node<'a>,
    by_y: Node<'a>,
    pub x: usize,
    pub y: usize,
}

impl<'a> Multiple<'a> {
    pub fn new(x: usize, y: usize) -> Multiple<'a> {
        Multiple {
            x,
            y,
            ..Default::default()
        }
    }
}

pub struct ByX;

impl_intrusive_node! {
    impl<'a> IntrusiveNode<'a> for ByX
    where
        type Elem = Multiple<'a>,
        node = by_x;
}

impl<'a> TreeOrd<'a, ByX> for Multiple<'a> {
    fn tree_cmp(&self, rhs: &Multiple<'a>) -> Ordering {
        self.x.cmp(&rhs.x)
    }
}

impl<'a> TreeOrd<'a, ByX> for usize {
    fn tree_cmp(&self, rhs: &Multiple<'a>) -> Ordering {
        self.cmp(&rhs.x)
    }
}

pub struct ByY;

intrusive_splay_tree::impl_intrusive_node! {
    impl<'a> IntrusiveNode<'a> for ByY
    where
        type Elem = Multiple<'a>,
        node = by_y;
}

impl<'a> TreeOrd<'a, ByY> for Multiple<'a> {
    fn tree_cmp(&self, rhs: &Multiple<'a>) -> Ordering {
        self.y.cmp(&rhs.y)
    }
}

impl<'a> TreeOrd<'a, ByY> for usize {
    fn tree_cmp(&self, rhs: &Multiple<'a>) -> Ordering {
        self.cmp(&rhs.y)
    }
}

pub fn trees_from_xs_and_ys<'a>(
    arena: &'a bumpalo::Bump,
    xs: Vec<usize>,
    ys: Vec<usize>,
    x: usize,
    y: usize,
) -> (SplayTree<'a, ByX>, SplayTree<'a, ByY>, bool, bool) {
    let min_len = min(xs.len(), ys.len());
    let mut xs = xs;
    let mut ys = ys;
    xs.truncate(min_len);
    ys.truncate(min_len);

    let x_in_xs = xs.contains(&x);
    let y_in_ys = ys.contains(&y);

    let mut by_x = SplayTree::<ByX>::default();
    let mut by_y = SplayTree::<ByY>::default();
    for (x, y) in xs.into_iter().zip(ys) {
        let m = arena.alloc(Multiple::new(x, y));
        by_x.insert(m);
        by_y.insert(m);
    }

    (by_x, by_y, x_in_xs, y_in_ys)
}
