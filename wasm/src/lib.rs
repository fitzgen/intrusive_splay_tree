#![no_std]
#![feature(core_intrinsics, lang_items)]

#[macro_use]
extern crate intrusive_splay_tree;

use core::cmp::Ordering;
use core::mem;
use core::ptr;

pub use intrusive_splay_tree::SplayTree;

// Need to provide a tiny `panic_fmt` lang-item implementation for `#![no_std]`.
// This implementation will translate panics into traps in the resulting
// WebAssembly.
#[lang = "panic_fmt"]
extern "C" fn panic_fmt(
    _args: ::core::fmt::Arguments,
    _file: &'static str,
    _line: u32
) -> ! {
    use core::intrinsics;
    unsafe {
        intrinsics::abort();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub u32);

#[derive(Debug)]
pub struct Monster<'a> {
    id: Id,
    health: u32,
    by_id_node: intrusive_splay_tree::Node<'a>,
    by_health_node: intrusive_splay_tree::Node<'a>,
}

pub struct MonstersById;

impl_intrusive_node! {
    impl<'a> IntrusiveNode<'a> for MonstersById
    where
        type Elem = Monster<'a>,
        node = by_id_node;
}

impl<'a> intrusive_splay_tree::TreeOrd<'a, MonstersById> for Monster<'a> {
    fn tree_cmp(&self, rhs: &Monster<'a>) -> Ordering {
        self.id.cmp(&rhs.id)
    }
}

pub struct MonstersByHealth;
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

impl<'a> intrusive_splay_tree::TreeOrd<'a, MonstersByHealth> for u32 {
    fn tree_cmp(&self, rhs: &Monster<'a>) -> Ordering {
        self.cmp(&rhs.health)
    }
}

impl<'a> intrusive_splay_tree::TreeOrd<'a, MonstersById> for Id {
    fn tree_cmp(&self, rhs: &Monster<'a>) -> Ordering {
        self.cmp(&rhs.id)
    }
}

extern "C" {
    fn alloc(n: usize) -> *mut u8;
}

#[no_mangle]
pub unsafe extern "C" fn new_id_tree() -> *mut SplayTree<'static, MonstersById> {
    let p = alloc(mem::size_of::<SplayTree<MonstersById>>());
    let p = p as *mut SplayTree<MonstersById>;
    ptr::write(p, SplayTree::default());
    p
}

#[no_mangle]
pub unsafe extern "C" fn new_health_tree() -> *mut SplayTree<'static, MonstersByHealth> {
    let p = alloc(mem::size_of::<SplayTree<MonstersByHealth>>());
    let p = p as *mut SplayTree<MonstersByHealth>;
    ptr::write(p, SplayTree::default());
    p
}

#[no_mangle]
pub unsafe extern "C" fn new_monster(
    id: u32,
    health: u32,
    by_id_tree: *mut SplayTree<'static, MonstersById>,
    by_health_tree: *mut SplayTree<'static, MonstersByHealth>,
) -> *const Monster<'static> {
    let p = alloc(mem::size_of::<Monster>());
    let p = p as *mut Monster<'static>;
    ptr::write(
        p,
        Monster {
            id: Id(id),
            health,
            by_id_node: Default::default(),
            by_health_node: Default::default(),
        },
    );
    let monster = &*p;
    (*by_id_tree).insert(monster);
    (*by_health_tree).insert(monster);
    monster
}

#[no_mangle]
pub unsafe extern "C" fn query_by_id(
    tree: *mut SplayTree<'static, MonstersById>,
    id: u32,
) -> *const Monster<'static> {
    (*tree).find(&Id(id)).map_or(ptr::null(), |m| m as *const _)
}

#[no_mangle]
pub unsafe extern "C" fn query_by_health(
    tree: *mut SplayTree<'static, MonstersByHealth>,
    health: u32,
) -> *const Monster<'static> {
    (*tree).find(&health).map_or(ptr::null(), |m| m as *const _)
}
