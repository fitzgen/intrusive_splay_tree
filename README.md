# intrusive_splay_tree

An intrusive, allocation-free [splay tree] implementation.

[![](https://docs.rs/intrusive_splay_tree/badge.svg)](https://docs.rs/intrusive_splay_tree/)
[![](https://img.shields.io/crates/v/intrusive_splay_tree.svg)](https://crates.io/crates/intrusive_splay_tree)
[![](https://img.shields.io/crates/d/intrusive_splay_tree.svg)](https://crates.io/crates/intrusive_splay_tree)
[![Travis CI Build Status](https://travis-ci.org/fitzgen/intrusive_splay_tree.svg?branch=master)](https://travis-ci.org/fitzgen/intrusive_splay_tree)

Splay trees are self-adjusting, meaning that operating on an element (for
example, doing a `find` or an `insert`) rebalances the tree in such a way
that the element becomes the root. This means that subsequent operations on
that element are *O(1)* as long as no other element is operated on in the
meantime.

### Implementation and Goals

* **Intrusive:** The space for the subtree pointers is stored *inside* the
element type. In non-intrusive trees, we would have a node type that
contains the subtree pointers and either a pointer to the element or we
would move the element into the node. The intrusive design inverts the
relationship, so that the elements hold the subtree pointers within
themselves.

* **Freedom from allocations and moves:** The intrusive design enables this
implementation to fully avoid both allocations and moving elements in
memory. Since the space for subtree pointers already exists in the element,
no allocation is necessary, just a handful of pointer writes. Therefore,
this implementation can be used in constrained environments that don't have
access to an allocator (e.g. some embedded devices or within a signal
handler) and with types that can't move in memory (e.g. `pthread_mutex_t`).

* **Small code size:** This implementation is geared towards small code
size, and uses trait objects internally to avoid the code bloat induced by
monomorphization. This implementation is suitable for targeting WebAssembly,
where code is downloaded over the network, and code bloat delays Web page
loading.

* **Nodes do not have parent pointers**: An intrusive node is only two words
in size: left and right sub tree pointers. There are no parent pointers,
which would require another word of overhead. To meet this goal, the
implementation uses the "top-down" variant of splay trees.

[splay tree]: https://en.wikipedia.org/wiki/Splay_tree
[paper]: http://www.cs.cmu.edu/~sleator/papers/self-adjusting.pdf

### Constraints

* **Elements within a tree must all have the same lifetime.** This means
that you must use something like the [`typed_arena`][arena] crate for
allocation, or be working with static data, etc.

* **Elements in an intrusive collections are inherently shared.** They are
always potentially aliased by the collection(s) they are in. In the other
direction, a particular intrusive collection only has a shared reference to
the element, since elements can both be in many intrusive collections at the
same time. Therefore, you cannot get a unique, mutable reference to an
element out of an intrusive splay tree. To work around this, you may need to
liberally use interior mutability, for example by leveraging `Cell`,
`RefCell`, and `Mutex`.

[arena]: https://crates.io/crates/typed-arena

### Example

This example defines a `Monster` type, where each of its instances live
within two intrusive trees: one ordering monsters by their name, and the
other ordering them by their health.

```rust
#[macro_use]
extern crate intrusive_splay_tree;
extern crate typed_arena;

use intrusive_splay_tree::SplayTree;

use std::cmp::Ordering;
use std::marker::PhantomData;

// We have a monster type, and we want to query monsters by both name and
// health.
#[derive(Debug)]
struct Monster<'a> {
    name: String,
    health: u64,

    // An intrusive node so we can put monsters in a tree to query by name.
    by_name_node: intrusive_splay_tree::Node<'a>,

    // Another intrusive node so we can put monsters in a second tree (at
    // the same time!) and query them by health.
    by_health_node: intrusive_splay_tree::Node<'a>,
}

// Define a type for trees where monsters are ordered by name.
struct MonstersByName;

// Implement `IntrusiveNode` for the `MonstersByName` tree, where the
// element type is `Monster` and the field in `Monster` that has this tree's
// intrusive node is `by_name`.
impl_intrusive_node! {
    impl<'a> IntrusiveNode<'a> for MonstersByName
    where
        type Elem = Monster<'a>,
        node = by_name_node;
}

// Define how to order `Monster`s within the `MonstersByName` tree by
// implementing `TreeOrd`.
impl<'a> intrusive_splay_tree::TreeOrd<'a, MonstersByName> for Monster<'a> {
    fn tree_cmp(&self, rhs: &Monster<'a>) -> Ordering {
        self.name.cmp(&rhs.name)
    }
}

// And do all the same things for trees where monsters are ordered by health...
struct MonstersByHealth;
impl_intrusive_node! {
    impl<'a> IntrusiveNode<'a> for MonstersByHealth
    where
        type Elem = Monster<'a>,
        node = by_health_node;
}
impl<'a> intrusive_splay_tree::TreeOrd<'a, MonstersByHealth> for Monster<'a> {
    fn tree_cmp(&self, rhs: &Monster<'a>) -> Ordering {
        self.health.cmp(&rhs.health)
    }
}

// We can also implement `TreeOrd` for other types, so that we can query the
// tree by these types. For example, we want to query the `MonstersByHealth`
// tree by some `u64` health value, and we want to query the `MonstersByName`
// tree by some `&str` name value.

impl<'a> intrusive_splay_tree::TreeOrd<'a, MonstersByHealth> for u64 {
    fn tree_cmp(&self, rhs: &Monster<'a>) -> Ordering {
        self.cmp(&rhs.health)
    }
}

impl<'a> intrusive_splay_tree::TreeOrd<'a, MonstersByName> for str {
    fn tree_cmp(&self, rhs: &Monster<'a>) -> Ordering {
        self.cmp(&rhs.name)
    }
}

impl<'a> Monster<'a> {
    /// The `Monster` constructor allocates `Monster`s in a typed arena, and
    /// inserts the new `Monster` in both trees.
    pub fn new(
        arena: &'a typed_arena::Arena<Monster<'a>>,
        name: String,
        health: u64,
        by_name_tree: &mut SplayTree<'a, MonstersByName>,
        by_health_tree: &mut SplayTree<'a, MonstersByHealth>
    ) -> &'a Monster<'a> {
        let monster = arena.alloc(Monster {
            name,
            health,
            by_name_node: Default::default(),
            by_health_node: Default::default(),
        });

        by_name_tree.insert(monster);
        by_health_tree.insert(monster);

        monster
    }
}

fn main() {
    // The arena that the monsters will live within.
    let mut arena = typed_arena::Arena::new();

    // The splay trees ordered by name and health respectively.
    let mut by_name_tree = SplayTree::default();
    let mut by_health_tree = SplayTree::default();

    // Now let's create some monsters, inserting them into the trees!

    Monster::new(
        &arena,
        "Frankenstein's Monster".into(),
        99,
        &mut by_name_tree,
        &mut by_health_tree,
    );

    Monster::new(
        &arena,
        "Godzilla".into(),
        2000,
        &mut by_name_tree,
        &mut by_health_tree,
    );

    Monster::new(
        &arena,
        "Vegeta".into(),
        9001,
        &mut by_name_tree,
        &mut by_health_tree,
    );

    // Query the `MonstersByName` tree by a name.

    let godzilla = by_name_tree.find("Godzilla").unwrap();
    assert_eq!(godzilla.name, "Godzilla");

    assert!(by_name_tree.find("Gill-Man").is_none());

    // Query the `MonstersByHealth` tree by a health.

    let vegeta = by_health_tree.find(&9001).unwrap();
    assert_eq!(vegeta.name, "Vegeta");

    assert!(by_health_tree.find(&0).is_none());
}
```

License: MPL-2.0
