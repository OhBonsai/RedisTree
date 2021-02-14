//! Composed of a root `Node` and a list of its child `Node`s.
//!
//! 1. Support adding and storing nodes only once on tree creation, in a
//! contiguous memory address.
//!
//! 2. Support adding and storing nodes one by one, in scattered memory
//! allocations.
//!
//! 3. Tuple notations for construction.
//!
//! 4. `tr()`,`-`,`/` notations for construction.
//!
//! 5. Can be converted to `RcNode` which has shared ownership.

use crate::TupleTree;

use crate::rust::*;

use super::{Data, Forest, IterMut, Node, NodeVec, heap};

/// Composed of a root `Node` and a list of its child `Node`s.
pub struct Tree<T>{
    pub(crate) root : NonNull<Node<T>>,
    pub(crate) mark : PhantomData<Node<T>>,
}

impl<T> Tree<T> {
    /// Creates a `Tree` containing only root node associated with given data.
    pub fn new( data: T ) -> Tree<T> {
        Tree {
            root: heap::make_node( Data::Scattered{ data, owner: NonNull::dangling() }),
            mark: PhantomData,
        }
    }

    /// Constructs tree from tuple notations.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Tree, tr};
    ///
    /// let tree = Tree::<i32>::from_tuple(( 0, (1,2), (3,4) ));
    /// assert_eq!( tree, tr(0) /(tr(1)/tr(2)) /(tr(3)/tr(4)) );
    /// assert_eq!( tree.to_string(), "0( 1( 2 ) 3( 4 ) )" );
    /// ```
    pub fn from_tuple<Tuple,Shape>( tuple: Tuple ) -> Self
        where Tuple: TupleTree<T,Shape>
    {
        let node_count = <Tuple as TupleTree<T,Shape>>::SIZE.descendants+1;
        let mut node_vec = NodeVec::new_raw_non_null( node_count );
        unsafe{ node_vec.as_mut().construct_tree( tuple )};

        Tree::from_node( unsafe{ node_vec.as_ref().non_null_node(0) })
    }

    pub(crate) fn into_data( mut self ) -> T {
        let value = self.root_mut_().data.replace( Data::None ).into_inner();
        mem::forget( self );
        value
    }

    pub(crate) fn from_node( mut root: NonNull<Node<T>> ) -> Tree<T> {
        unsafe{ root.as_mut().up = None; }
        Tree{ root, mark: PhantomData }
    }

    /// Reference of the root node.
    pub fn root( &self ) -> &Node<T> { unsafe{ &*self.root.as_ptr() }}

    /// Mutable reference of the root node.
    pub fn root_mut( &mut self ) -> Pin<&mut Node<T>> { unsafe{ Pin::new_unchecked( self.root_mut_() )}}

    pub(crate) fn root_mut_( &mut self ) -> &mut Node<T> { unsafe{ &mut *self.root.as_ptr() }}

    /// Provides a forward iterator over child `Node`s with mutable references.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let mut tree = Tree::<i32>::from_tuple(( 0, 1, 2, 3 ));
    /// tree.iter_mut().for_each( |mut child| *child.data_mut() *= 10 );
    /// assert_eq!( tree.to_string(), "0( 10 20 30 )" );
    /// ```
    pub fn iter_mut<'a, 's:'a>( &'s mut self ) -> IterMut<'a,T> { self.root_mut_().iter_mut() }

    /// Adds the tree as the first child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    /// let mut tree = Tree::new(0);
    /// tree.push_front( Tree::new(1) );
    /// assert_eq!( tree.to_string(), "0( 1 )" );
    /// tree.push_front( Tree::new(2) );
    /// assert_eq!( tree.to_string(), "0( 2 1 )" );
    /// ```
    pub fn push_front( &mut self, tree: Tree<T> ) {
        self.root_mut_().push_front( tree );
    }

    /// Adds the tree as the last child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    /// let mut tree = Tree::new(0);
    /// tree.push_back( Tree::new(1) );
    /// assert_eq!( tree.to_string(), "0( 1 )" );
    /// tree.push_back( Tree::new(2) );
    /// assert_eq!( tree.to_string(), "0( 1 2 )" );
    /// ```
    pub fn push_back( &mut self, tree: Tree<T> ) {
        self.root_mut_().push_back( tree );
    }

    /// Adds all the forest's trees at front of children list.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Forest, Tree};
    ///
    /// let mut tree = Tree::new(0);
    /// tree.push_back( Tree::new(1) );
    /// tree.push_back( Tree::new(2) );
    /// let mut forest = Forest::new();
    /// forest.push_back( Tree::new(3) );
    /// forest.push_back( Tree::new(4) );
    /// tree.prepend( forest );
    /// assert_eq!( tree.to_string(), "0( 3 4 1 2 )" );
    /// ```
    pub fn prepend( &mut self, forest: Forest<T> ) {
        self.root_mut_().prepend( forest );
    }

    /// Adds all the forest's trees at back of children list.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Forest, Tree};
    ///
    /// let mut tree = Tree::new(0);
    /// tree.push_back( Tree::new(1) );
    /// tree.push_back( Tree::new(2) );
    /// let mut forest = Forest::new();
    /// forest.push_back( Tree::new(3) );
    /// forest.push_back( Tree::new(4) );
    /// tree.root_mut().append( forest );
    /// assert_eq!( tree.to_string(), "0( 1 2 3 4 )" );
    /// ```
    pub fn append( &mut self, forest: Forest<T> ) {
        self.root_mut_().append( forest );
    }

    /// Removes and returns the given `Tree`'s children.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Forest, Tree};
    ///
    /// let mut tree = Tree::new(0);
    /// tree.push_back( Tree::new(1) );
    /// tree.push_back( Tree::new(2) );
    /// let forest = tree.abandon();
    /// assert_eq!( forest.to_string(), "( 1 2 )" );
    /// assert_eq!( tree, Tree::new(0) );
    /// ```
    pub fn abandon( &mut self ) -> Forest<T> {
        let new_root = heap::make_node( Data::Scattered{
            data  : self.root_mut_().data.take(),
            owner : NonNull::dangling(),
        });
        let forest = Forest::from_node( self.root );
        self.root = new_root;
        forest
    }

    /// Removes and returns the first child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let mut tree = Tree::new(0);
    /// tree.push_back( Tree::new(1) );
    /// tree.push_back( Tree::new(2) );
    /// assert_eq!( tree.to_string(), "0( 1 2 )" );
    /// assert_eq!( tree.pop_front(), Some( Tree::new(1) ));
    /// assert_eq!( tree.to_string(), "0( 2 )" );
    /// assert_eq!( tree.pop_front(), Some( Tree::new(2) ));
    /// assert_eq!( tree.to_string(), "0" );
    /// assert_eq!( tree.pop_front(), None );
    /// ```
    pub fn pop_front( &mut self ) -> Option<Tree<T>> { self.root_mut_().pop_front() }

    /// Removes and returns the last child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let mut tree = Tree::new(0);
    /// tree.push_back( Tree::new(1) );
    /// tree.push_back( Tree::new(2) );
    /// assert_eq!( tree.to_string(), "0( 1 2 )" );
    /// assert_eq!( tree.pop_back(), Some( Tree::new(2) ));
    /// assert_eq!( tree.to_string(), "0( 1 )" );
    /// assert_eq!( tree.pop_back(), Some( Tree::new(1) ));
    /// assert_eq!( tree.to_string(), "0" );
    /// assert_eq!( tree.pop_back(), None );
    /// ```
    pub fn pop_back( &mut self ) -> Option<Tree<T>> { self.root_mut_().pop_back() }

    /// Returns a mutable reference to the first child of this node,
    /// or None if it has no child.
    pub fn front_mut( &mut self ) -> Option<Pin<&mut Node<T>>> { self.root_mut_().front_mut() }

    /// Returns a mutable reference to the last child of this node,
    /// or None if it has no child.
    pub fn back_mut( &mut self ) -> Option<Pin<&mut Node<T>>> { self.root_mut_().back_mut() }
}

impl<T:Clone> Clone for Tree<T> {
    fn clone( &self ) -> Self {
        self.root().deep_clone()
    }
}

impl<T> Deref for Tree<T> {
    type Target = Node<T>;

    fn deref( &self ) -> &Self::Target { self.root() }
}

impl<T> Drop for Tree<T> {
    fn drop( &mut self ) {
        while let Some(_) = self.root_mut_().pop_front() {}
        heap::drop_node( self.root );
    }
}

impl_debug_display_for_collection!( Tree, root() );
impl_order_relations_for_collection!( Tree, root() );
impl_hash_for_collection!( Tree, root() );

#[cfg( test )]
mod tests {
    use super::*;

    #[test] fn piled_tree_from_tuple() {
        let tuple = ( 0, (1,2,3), (4,5,6) );
        let piled = Tree::<i32>::from_tuple( tuple );
        assert_eq!( piled.to_string(), "0( 1( 2 3 ) 4( 5 6 ) )" );
    }
}

#[cfg( miri )]
mod miri_tests {
    #[test] fn iter_mut() {
        use crate::Tree;

        let mut tree = Tree::<i32>::from_tuple(( 0, 1, 2, 3 ));
        tree.iter_mut().for_each( |mut child| *child.data_mut() *= 10 );
        assert_eq!( tree.to_string(), "0( 10 20 30 )" );
    }

    #[test] fn push_front() {
        use crate::Tree;

        let mut tree = Tree::new(0);
        tree.push_front( Tree::new(1) );
        assert_eq!( tree.to_string(), "0( 1 )" );
        tree.push_front( Tree::new(2) );
        assert_eq!( tree.to_string(), "0( 2 1 )" );
    }

    #[test] fn push_back() {
        use crate::Tree;

        let mut tree = Tree::new(0);
        tree.push_back( Tree::new(1) );
        assert_eq!( tree.to_string(), "0( 1 )" );
        tree.push_back( Tree::new(2) );
        assert_eq!( tree.to_string(), "0( 1 2 )" );
    }

    #[test] fn prepend() {
        use crate::{Forest, Tree};

        let mut tree = Tree::new(0);
        tree.push_back( Tree::new(1) );
        tree.push_back( Tree::new(2) );
        let mut forest = Forest::new();
        forest.push_back( Tree::new(3) );
        forest.push_back( Tree::new(4) );
        tree.prepend( forest );
        assert_eq!( tree.to_string(), "0( 3 4 1 2 )" );
    }

    #[test] fn append() {
        use crate::{Forest, Tree};

        let mut tree = Tree::new(0);
        tree.push_back( Tree::new(1) );
        tree.push_back( Tree::new(2) );
        let mut forest = Forest::new();
        forest.push_back( Tree::new(3) );
        forest.push_back( Tree::new(4) );
        tree.root_mut().append( forest );
        assert_eq!( tree.to_string(), "0( 1 2 3 4 )" );
    }

    #[test] fn abandon() {
        use crate::Tree;

        let mut tree = Tree::new(0);
        tree.push_back( Tree::new(1) );
        tree.push_back( Tree::new(2) );
        let forest = tree.abandon();
        assert_eq!( forest.to_string(), "( 1 2 )" );
        assert_eq!( tree, Tree::new(0) );
    }

    #[test] fn pop_front() {
        use crate::Tree;

        let mut tree = Tree::new(0);
        tree.push_back( Tree::new(1) );
        tree.push_back( Tree::new(2) );
        assert_eq!( tree.to_string(), "0( 1 2 )" );
        assert_eq!( tree.pop_front(), Some( Tree::new(1) ));
        assert_eq!( tree.to_string(), "0( 2 )" );
        assert_eq!( tree.pop_front(), Some( Tree::new(2) ));
        assert_eq!( tree.to_string(), "0" );
        assert_eq!( tree.pop_front(), None );
    }

    #[test] fn pop_back() {
        use crate::Tree;

        let mut tree = Tree::new(0);
        tree.push_back( Tree::new(1) );
        tree.push_back( Tree::new(2) );
        assert_eq!( tree.to_string(), "0( 1 2 )" );
        assert_eq!( tree.pop_back(), Some( Tree::new(2) ));
        assert_eq!( tree.to_string(), "0( 1 )" );
        assert_eq!( tree.pop_back(), Some( Tree::new(1) ));
        assert_eq!( tree.to_string(), "0" );
        assert_eq!( tree.pop_back(), None );
    }
    #[test] fn from_tuple() {
        use crate::{Tree, tr};

        let tree = Tree::<i32>::from_tuple(( 0, (1,2), (3,4) ));
        assert_eq!( tree, tr(0) /(tr(1)/tr(2)) /(tr(3)/tr(4)) );
        assert_eq!( tree.to_string(), "0( 1( 2 ) 3( 4 ) )" );
    }
}
