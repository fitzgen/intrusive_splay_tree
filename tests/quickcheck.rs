#[macro_use]
extern crate intrusive_splay_tree;

#[macro_use]
extern crate quickcheck;

mod single;

use intrusive_splay_tree::{IntrusiveNode, Node, SplayTree, TreeOrd};
use single::{Single, SingleTree};
use std::cmp::{min, Ordering};
use std::iter::FromIterator;
use std::marker::PhantomData;

quickcheck! {
    fn find(xs: Vec<usize>, x: usize) -> bool {
        let x_in_xs = xs.contains(&x);

        let arena = bumpalo::Bump::new();

        let mut tree = SplayTree::<SingleTree>::from_iter(
            xs.into_iter()
                .map(|x| &*arena.alloc(Single::new(x)))
        );

        if let Some(c) = tree.find(&x) {
            x_in_xs && c.value == x
        } else {
            !x_in_xs
        }
    }

    fn remove(xs: Vec<usize>, x: usize) -> bool {
        let x_in_xs = xs.contains(&x);

        let arena = bumpalo::Bump::new();

        let mut tree = SplayTree::<SingleTree>::from_iter(
            xs.into_iter()
                .map(|x| &*arena.alloc(Single::new(x)))
        );

        if let Some(removed) = tree.remove(&x) {
            x_in_xs && removed.value == x && tree.find(&x).is_none()
        } else {
            !x_in_xs
        }
    }

    fn insert(xs: Vec<usize>, x: usize) -> bool {
        let x_in_xs = xs.contains(&x);

        let arena = bumpalo::Bump::new();

        let mut tree = SplayTree::<SingleTree>::from_iter(
            xs.into_iter()
                .map(|x| &*arena.alloc(Single::new(x)))
        );

        let is_new_entry = tree.insert(arena.alloc(Single::new(x)));
        ((is_new_entry && !x_in_xs) || x_in_xs) && tree.find(&x).map_or(false, |c| c.value == x)
    }

    fn tree_min(xs: Vec<usize>) -> bool {
        let min = xs.iter().copied().min();

        let arena = bumpalo::Bump::new();

        let mut tree = SplayTree::<SingleTree>::from_iter(
            xs.into_iter()
                .map(|x| &*arena.alloc(Single::new(x)))
        );

        tree.min().map(|s| s.value) == min
    }

    fn tree_max(xs: Vec<usize>) -> bool {
        let max = xs.iter().copied().max();

        let arena = bumpalo::Bump::new();

        let mut tree = SplayTree::<SingleTree>::from_iter(
            xs.into_iter()
                .map(|x| &*arena.alloc(Single::new(x)))
        );

        tree.max().map(|s| s.value) == max
    }

    fn pop_min(xs: Vec<usize>) -> bool {
        if xs.is_empty() {
            return true;
        }

        let arena = bumpalo::Bump::new();

        let mut tree = SplayTree::<SingleTree>::from_iter(
            xs.into_iter()
                .map(|x| &*arena.alloc(Single::new(x)))
        );

        let mut prev_min = tree.pop_min().unwrap().value;
        while let Some(n) = tree.pop_min() {
            if n.value < prev_min {
                return false;
            }
            prev_min = n.value;
        }

        true
    }

    fn pop_max(xs: Vec<usize>) -> bool {
        if xs.is_empty() {
            return true;
        }

        let arena = bumpalo::Bump::new();

        let mut tree = SplayTree::<SingleTree>::from_iter(
            xs.into_iter()
                .map(|x| &*arena.alloc(Single::new(x)))
        );

        let mut prev_max = tree.pop_max().unwrap().value;
        while let Some(n) = tree.pop_max() {
            if n.value > prev_max {
                return false;
            }
            prev_max = n.value;
        }

        true
    }
}

#[derive(Debug, Default)]
struct Multiple<'a> {
    by_x: intrusive_splay_tree::Node<'a>,
    by_y: intrusive_splay_tree::Node<'a>,
    x: usize,
    y: usize,
}

impl<'a> Multiple<'a> {
    fn new(x: usize, y: usize) -> Multiple<'a> {
        Multiple {
            x,
            y,
            ..Default::default()
        }
    }
}

struct ByX<'a>(PhantomData<&'a Multiple<'a>>);

impl_intrusive_node! {
    impl<'a> IntrusiveNode<'a> for ByX<'a>
    where
        type Elem = Multiple<'a>,
        node = by_x;
}

impl<'a> TreeOrd<'a, ByX<'a>> for Multiple<'a> {
    fn tree_cmp(&self, rhs: &Multiple<'a>) -> Ordering {
        self.x.cmp(&rhs.x)
    }
}

impl<'a> TreeOrd<'a, ByX<'a>> for usize {
    fn tree_cmp(&self, rhs: &Multiple<'a>) -> Ordering {
        self.cmp(&rhs.x)
    }
}

struct ByY<'a>(PhantomData<&'a Multiple<'a>>);

unsafe impl<'a> IntrusiveNode<'a> for ByY<'a> {
    type Elem = Multiple<'a>;

    fn elem_to_node(elem: &'a Self::Elem) -> &'a Node<'a> {
        &elem.by_y
    }

    unsafe fn node_to_elem(node: &'a Node<'a>) -> &'a Self::Elem {
        let offset = {
            let m = Multiple::default();
            let node = &m.by_y as *const _ as usize;
            let m = &m as *const _ as usize;
            node - m
        };
        let node = node as *const _ as *const u8;
        let elem = node.offset(-(offset as isize)) as *const Self::Elem;
        &*elem
    }
}

impl<'a> TreeOrd<'a, ByY<'a>> for Multiple<'a> {
    fn tree_cmp(&self, rhs: &Multiple<'a>) -> Ordering {
        self.y.cmp(&rhs.y)
    }
}

impl<'a> TreeOrd<'a, ByY<'a>> for usize {
    fn tree_cmp(&self, rhs: &Multiple<'a>) -> Ordering {
        self.cmp(&rhs.y)
    }
}

fn trees_from_xs_and_ys<'a>(
    arena: &'a bumpalo::Bump,
    xs: Vec<usize>,
    ys: Vec<usize>,
    x: usize,
    y: usize,
) -> (SplayTree<'a, ByX<'a>>, SplayTree<'a, ByY<'a>>, bool, bool) {
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

quickcheck! {
    fn multiple_find(xs: Vec<usize>, ys: Vec<usize>, x: usize, y: usize) -> bool {
        let arena = bumpalo::Bump::new();
        let (mut by_x, mut by_y, x_in_xs, y_in_ys) = trees_from_xs_and_ys(&arena, xs, ys, x, y);

        let by_x_ok = if let Some(m) = by_x.find(&x) {
            x_in_xs && m.x == x
        } else {
            !x_in_xs
        };

        let by_y_ok = if let Some(m) = by_y.find(&y) {
            y_in_ys && m.y == y
        } else {
            !y_in_ys
        };

        by_x_ok && by_y_ok
    }

    fn multiple_remove(xs: Vec<usize>, ys: Vec<usize>, x: usize, y: usize) -> bool {
        let arena = bumpalo::Bump::new();
        let (mut by_x, mut by_y, x_in_xs, y_in_ys) = trees_from_xs_and_ys(&arena, xs, ys, x, y);

        let by_x_ok = if let Some(m) = by_x.remove(&x) {
            x_in_xs && m.x == x
        } else {
            !x_in_xs
        };
        let by_x_ok = by_x_ok && by_x.find(&x).is_none();

        let by_y_ok = if let Some(m) = by_y.remove(&y) {
            y_in_ys && m.y == y
        } else {
            !y_in_ys
        };
        let by_y_ok = by_y_ok && by_y.find(&y).is_none();

        by_x_ok && by_y_ok
    }

    fn multiple_insert(xs: Vec<usize>, ys: Vec<usize>, x: usize, y: usize) -> bool {
        let arena = bumpalo::Bump::new();
        let (mut by_x, mut by_y, x_in_xs, y_in_ys) = trees_from_xs_and_ys(&arena, xs, ys, x, y);

        let elem = arena.alloc(Multiple::new(x, y));
        let x_is_new = by_x.insert(elem);
        let y_is_new = by_y.insert(elem);

        ((x_is_new && !x_in_xs) || x_in_xs) && by_x.find(&x).map_or(false, |m| m.x == x) &&
        ((y_is_new && !y_in_ys) || y_in_ys) && by_y.find(&y).map_or(false, |m| m.y == y)
    }
}
