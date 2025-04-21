#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![no_std]

mod internal;
mod node;

pub use node::Node;

use core::cmp;
use core::fmt;
use core::iter;
use core::marker::PhantomData;

/// Defines how to get the intrusive node from a particular kind of
/// `SplayTree`'s element type.
///
/// Don't implement this by hand -- doing so is both boring and dangerous!
/// Instead, use the `impl_intrusive_node!` macro.
pub unsafe trait IntrusiveNode<'a>
where
    Self: Sized,
{
    /// The element struct type that contains a node for this tree.
    type Elem: TreeOrd<'a, Self>;

    /// Get the node for this tree from the given element.
    fn elem_to_node(elem: &'a Self::Elem) -> &'a Node<'a>;

    /// Get the element for this node (by essentially doing `offsetof` the
    /// node's field).
    ///
    /// ## Safety
    ///
    /// Given a node inside a different element type, or a node for a different
    /// tree within the same element type, this method will result in memory
    /// unsafety.
    #[doc(hidden)]
    unsafe fn node_to_elem(node: &'a Node<'a>) -> &'a Self::Elem;
}

/// Implement `IntrusiveNode` for a particular kind of `SplayTree` and its
/// element type.
#[macro_export]
macro_rules! impl_intrusive_node {
    (
        impl< $($typarams:tt),* >
            IntrusiveNode<$intrusive_node_lifetime:tt>
            for $tree:ty
        where
            type Elem = $elem:ty ,
            node = $node:ident ;
    ) => {
        unsafe impl< $( $typarams )* > $crate::IntrusiveNode<$intrusive_node_lifetime> for $tree {
            type Elem = $elem;

            fn elem_to_node(
                elem: & $intrusive_node_lifetime Self::Elem
            ) -> & $intrusive_node_lifetime $crate::Node< $intrusive_node_lifetime > {
                &elem.$node
            }

            unsafe fn node_to_elem(
                node: & $intrusive_node_lifetime $crate::Node< $intrusive_node_lifetime >
            ) -> & $intrusive_node_lifetime Self::Elem {
                let node = core::ptr::with_exposed_provenance::<u8>(node as *const _ as usize);

                let offset = core::mem::offset_of!(Self::Elem, $node);
                let offset = isize::try_from(offset).unwrap();
                let neg_offset = offset.checked_neg().unwrap();

                let elem = node.offset(neg_offset);
                let elem = elem.cast::<Self::Elem>();

                elem.as_ref().unwrap()
            }
        }
    }
}

/// A total ordering between the `Self` type and the tree's element type
/// `T::Elem`.
///
/// Different from `Ord` in that it allows `Self` and `T::Elem` to be distinct
/// types, so that you can query a splay tree without fully constructing its
/// element type.
pub trait TreeOrd<'a, T: IntrusiveNode<'a>> {
    /// What is the ordering relationship between `self` and the given tree
    /// element?
    fn tree_cmp(&self, elem: &'a T::Elem) -> cmp::Ordering;
}

struct Query<'a, 'b, K, T>
where
    T: 'a + IntrusiveNode<'a>,
    K: 'b + ?Sized + TreeOrd<'a, T>,
{
    key: &'b K,
    _phantom: PhantomData<&'a T>,
}

impl<'a, 'b, K, T> Query<'a, 'b, K, T>
where
    T: IntrusiveNode<'a>,
    K: 'b + ?Sized + TreeOrd<'a, T>,
{
    #[inline]
    fn new(key: &'b K) -> Query<'a, 'b, K, T> {
        Query {
            key,
            _phantom: PhantomData,
        }
    }
}

impl<'a, 'b, K, T> internal::CompareToNode<'a> for Query<'a, 'b, K, T>
where
    T: 'a + IntrusiveNode<'a>,
    T::Elem: 'a,
    K: 'b + ?Sized + TreeOrd<'a, T>,
{
    #[inline]
    unsafe fn compare_to_node(&self, node: &'a Node<'a>) -> cmp::Ordering {
        let val = T::node_to_elem(node);
        self.key.tree_cmp(val)
    }
}

/// An intrusive splay tree.
///
/// The tree is parameterized by some marker type `T` whose `IntrusiveNode`
/// implementation defines:
///
/// * the element type contained in this tree: `T::Elem`,
/// * how to get the intrusive node for this tree within an element,
/// * and how to get the container element from a given intrusive node for this
/// tree.
pub struct SplayTree<'a, T>
where
    T: IntrusiveNode<'a>,
    T::Elem: 'a,
{
    tree: internal::SplayTree<'a>,
    _phantom: PhantomData<&'a T::Elem>,
}

impl<'a, T> Default for SplayTree<'a, T>
where
    T: 'a + IntrusiveNode<'a>,
    T::Elem: 'a,
{
    #[inline]
    fn default() -> SplayTree<'a, T> {
        SplayTree {
            tree: internal::SplayTree::default(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T> fmt::Debug for SplayTree<'a, T>
where
    T: 'a + IntrusiveNode<'a>,
    T::Elem: 'a + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let set = &mut f.debug_set();
        self.walk(|x| {
            set.entry(x);
        });
        set.finish()
    }
}

impl<'a, T> Extend<&'a T::Elem> for SplayTree<'a, T>
where
    T: 'a + IntrusiveNode<'a>,
{
    #[inline]
    fn extend<I: IntoIterator<Item = &'a T::Elem>>(&mut self, iter: I) {
        for x in iter {
            self.insert(x);
        }
    }
}

impl<'a, T> iter::FromIterator<&'a T::Elem> for SplayTree<'a, T>
where
    T: 'a + IntrusiveNode<'a>,
    T::Elem: fmt::Debug,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = &'a T::Elem>>(iter: I) -> Self {
        let mut me = SplayTree::default();
        me.extend(iter);
        me
    }
}

impl<'a, T> SplayTree<'a, T>
where
    T: 'a + IntrusiveNode<'a>,
{
    /// Construct a new, empty tree.
    #[inline]
    pub const fn new() -> Self {
        Self {
            tree: internal::SplayTree::new(),
            _phantom: PhantomData,
        }
    }

    /// Is this tree empty?
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// Get a reference to the root element, if any exists.
    pub fn root(&self) -> Option<&'a T::Elem> {
        self.tree.root().map(|r| unsafe { T::node_to_elem(r) })
    }

    /// Find an element in the tree.
    ///
    /// This operation will splay the queried element to the root of the tree.
    ///
    /// The `key` must be of a type that implements `TreeOrd` for this tree's
    /// `T` type. The element type `T::Elem` must always implement `TreeOrd<T>`,
    /// so you can search the tree by element. You can also implement
    /// `TreeOrd<T>` for additional key types. This allows you to search the
    /// tree without constructing a full element.
    #[inline]
    pub fn find<K>(&mut self, key: &K) -> Option<&'a T::Elem>
    where
        K: ?Sized + TreeOrd<'a, T>,
    {
        unsafe {
            let query: Query<_, T> = Query::new(key);
            self.tree.find(&query).map(|node| T::node_to_elem(node))
        }
    }

    /// Insert a new element into this tree.
    ///
    /// Returns `true` if the element was inserted into the tree.
    ///
    /// Returns `false` if there was already an element in the tree for which
    /// `TreeOrd` returned `Ordering::Equal`. In this case, the extant element
    /// is left in the tree, and `elem` is not inserted.
    ///
    /// This operation will splay the inserted element to the root of the tree.
    ///
    /// It is a logic error to insert an element that is already inserted in a
    /// `T` tree.
    ///
    /// ## Panics
    ///
    /// If `debug_assertions` are enabled, then this function may panic if
    /// `elem` is already in a `T` tree. If `debug_assertions` are not defined,
    /// the behavior is safe, but unspecified.
    #[inline]
    pub fn insert(&mut self, elem: &'a T::Elem) -> bool {
        // To satisfy MIRI, we need to expose provenance of element added to the
        // tree, so that when we query the tree and go from node-to-elem, we can
        // use this exposed provenance. This is because, while the lifetimes
        // ensure that the element remains borrowed while inserted in the tree,
        // we don't have a good way to plumb through a pointer with the original
        // element's borrowed provenance through to all node-to-elem
        // conversions.
        let _ = (elem as *const T::Elem).expose_provenance();

        unsafe {
            let query: Query<_, T> = Query::new(elem);
            let node = T::elem_to_node(elem);
            self.tree.insert(&query, node)
        }
    }

    /// Find and remove an element from the tree.
    ///
    /// If a matching element is found and removed, then `Some(removed_element)`
    /// is returned. Otherwise `None` is returned.
    ///
    /// The `key` must be of a type that implements `TreeOrd` for this tree's
    /// `T` type. The element type `T::Elem` must always implement `TreeOrd<T>`,
    /// so you can remove an element directly. You can also implement
    /// `TreeOrd<T>` for additional key types. This allows you to search the
    /// tree without constructing a full element, and remove the element that
    /// matches the given key, if any.
    #[inline]
    pub fn remove<K>(&mut self, key: &K) -> Option<&'a T::Elem>
    where
        K: ?Sized + TreeOrd<'a, T>,
    {
        unsafe {
            let query: Query<_, T> = Query::new(key);
            self.tree.remove(&query).map(|node| T::node_to_elem(node))
        }
    }

    /// Pop the root element from the tree.
    ///
    /// If the tree has a root, it is removed and `Some(root)` is
    /// returned. Otherwise, `None` is returned.
    #[inline]
    pub fn pop_root(&mut self) -> Option<&'a T::Elem> {
        unsafe { self.tree.pop_root().map(|node| T::node_to_elem(node)) }
    }

    /// Get the minimum element in the tree.
    ///
    /// If the tree is non-empty, then the minimum element is splayed to the
    /// root and `Some(min_elem)` is returned. Otherwise, `None` is returned.
    #[inline]
    pub fn min(&mut self) -> Option<&'a T::Elem> {
        self.tree.min().map(|node| unsafe { T::node_to_elem(node) })
    }

    /// Pop the minimum element from the tree.
    ///
    /// If the tree is non-empty, then the minimum element is removed and
    /// `Some(_)` is returned. Otherwise, `None` is returned.
    #[inline]
    pub fn pop_min(&mut self) -> Option<&'a T::Elem> {
        unsafe { self.tree.pop_min().map(|node| T::node_to_elem(node)) }
    }

    /// Get the maximum element in the tree.
    ///
    /// If the tree is non-empty, then the maximum element is splayed to the
    /// root and `Some(max_elem)` is returned. Otherwise, `None` is returned.
    #[inline]
    pub fn max(&mut self) -> Option<&'a T::Elem> {
        self.tree.max().map(|node| unsafe { T::node_to_elem(node) })
    }

    /// Pop the maximum element from the tree.
    ///
    /// If the tree is non-empty, then the maximum element is removed and
    /// `Some(_)` is returned. Otherwise, `None` is returned.
    #[inline]
    pub fn pop_max(&mut self) -> Option<&'a T::Elem> {
        unsafe { self.tree.pop_max().map(|node| T::node_to_elem(node)) }
    }

    /// Walk the tree in order.
    ///
    /// The `C` type controls whether iteration should continue, or break and
    /// return a `C::Result` value. You can use `()` as `C`, and that always
    /// continues iteration. Using `Result<(), E>` as `C` allows you to halt
    /// iteration on error, and propagate the error value. Using `Option<T>` as
    /// `C` allows you to search for some value, halt iteration when its found,
    /// and return it.
    #[inline]
    pub fn walk<F, C>(&self, mut f: F) -> Option<C::Result>
    where
        F: FnMut(&'a T::Elem) -> C,
        C: WalkControl,
    {
        let mut result = None;
        self.tree.walk(&mut |node| unsafe {
            let elem = T::node_to_elem(node);
            result = f(elem).should_break();
            result.is_none()
        });
        result
    }
}

/// A trait that guides whether `SplayTree::walk` should continue or break, and
/// what the return value is.
pub trait WalkControl {
    /// The result type that is returned when we break.
    type Result;

    /// If iteration should halt, return `Some`. If iteration should continue,
    /// return `None`.
    fn should_break(self) -> Option<Self::Result>;
}

impl WalkControl for () {
    type Result = ();

    fn should_break(self) -> Option<()> {
        None
    }
}

impl<T> WalkControl for Option<T> {
    type Result = T;

    fn should_break(mut self) -> Option<T> {
        self.take()
    }
}

impl<E> WalkControl for Result<(), E> {
    type Result = E;

    fn should_break(self) -> Option<E> {
        self.err()
    }
}
