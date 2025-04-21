use crate::multiple::{trees_from_xs_and_ys, Multiple};
use crate::single::{Single, SingleTree};
use intrusive_splay_tree::SplayTree;
use std::iter::FromIterator;

/// Like the `quickcheck::quickcheck` macro, but limits the number of tests run
/// through MIRI, since MIRI execution is so slow.
#[macro_export]
macro_rules! quickcheck {
    (
        $(
            $(#[$m:meta])*
            fn $fn_name:ident($($arg_name:ident : $arg_ty:ty),*) -> $ret:ty {
                $($code:tt)*
            }
        )*
    ) => {
        $(
            #[test]
            $(#[$m])*
            fn $fn_name() {
                fn prop($($arg_name: $arg_ty),*) -> $ret {
                    $($code)*
                }

                let mut qc = ::quickcheck::QuickCheck::new();

                // Use the `QUICKCHECK_TESTS` environment variable from
                // compiletime to avoid violating MIRI's isolation by looking at
                // the runtime environment variable.
                let tests = option_env!("QUICKCHECK_TESTS").and_then(|s| s.parse().ok());

                // Limit quickcheck tests under MIRI, since they are otherwise
                // super slow.
                #[cfg(miri)]
                let tests = tests.or(Some(25));

                if let Some(tests) = tests {
                    eprintln!("Executing at most {} quickchecks", tests);
                    qc = qc.tests(tests);
                }

                qc.quickcheck(prop as fn($($arg_ty),*) -> $ret);
            }
        )*
    };
}

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

    fn pop_root(xs: Vec<usize>) -> bool {
        let arena = bumpalo::Bump::new();

        let mut tree = SplayTree::<SingleTree>::from_iter(
            xs.into_iter()
                .map(|x| &*arena.alloc(Single::new(x)))
        );

        let root = tree.root().map(|n| n.value);
        tree.pop_root().map(|n| n.value) == root
    }

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
