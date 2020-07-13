mod single;

use intrusive_splay_tree::SplayTree;
use single::{Single, SingleTree};
use std::panic;

#[test]
#[cfg(debug_assertions)]
fn inserting_already_inserted_panics_in_debug() {
    let result = panic::catch_unwind(panic::AssertUnwindSafe(move || {
        let arena = bumpalo::Bump::new();
        let mut tree = SplayTree::<SingleTree>::default();
        let elems = (0..3)
            .map(|x| arena.alloc(Single::new(x)))
            .collect::<Vec<_>>();

        for e in elems.iter() {
            tree.insert(e);
        }
        for e in elems.iter() {
            tree.insert(e);
        }
    }));
    assert!(result.is_err());
}
