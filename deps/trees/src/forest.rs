//! Composed of a list of `Node`s as its children.
//!
//! 1. Support adding and storing nodes only once on tree creation, in a
//! contiguous memory address.
//!
//! 2. Support adding and storing nodes one by one, in scattered memory
//! allocations.
//!
//! 3. Tuple notations for construction.
//!
//! 4. `fr()`,`-`,`/` notations for construction.

use super::heap;
use super::{Tree, Node, Data, Iter, IterMut};
use super::NodeVec;
use crate::{Size, TupleForest};

use crate::rust::*;

/// List of `Node`s as its children.
pub struct Forest<T> {
    root : NonNull<Node<T>>,
    mark : PhantomData<Node<T>>,
}

impl<T> Forest<T> {
    pub(crate) fn root_( &self ) -> &Node<T> { unsafe{ &*self.root.as_ptr() }}
    pub(crate) fn root_mut_( &mut self ) -> &mut Node<T> { unsafe{ &mut *self.root.as_ptr() }}

    /// Makes an empty `Forest`.
    pub fn new() -> Forest<T> {
        Forest::from_node( heap::make_node( Data::ScatteredNone{ owner: NonNull::dangling() }))
    }

    /// Construct forest from tuple notations.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Forest, tr};
    ///
    /// let forest = Forest::<i32>::from_tuple(( 0, (1,2), (3,4) ));
    /// assert_eq!( forest, -tr(0) -tr(1)/tr(2) -tr(3)/tr(4) );
    /// assert_eq!( forest.to_string(), "( 0 1( 2 ) 3( 4 ) )" );
    /// ```
    pub fn from_tuple<Tuple,Shape>( tuple: Tuple ) -> Self
        where Tuple : TupleForest<T,Shape>
    {
        let node_count = <Tuple as TupleForest<T,Shape>>::SIZE.descendants + 1;
        let mut node_vec = NodeVec::new_raw_non_null( node_count );
        unsafe{ node_vec.as_mut().construct_forest( tuple )};

        Forest::from_node( unsafe{ node_vec.as_ref().non_null_node(0) })
    }

    pub(crate) fn from_node( root: NonNull<Node<T>> ) -> Forest<T> {
        Forest{ root, mark: PhantomData }
    }

    pub(crate) fn set_up( &mut self, parent: &mut Node<T> ) {
        self.iter_mut()
            .map( |node| unsafe{ Pin::get_unchecked_mut( node )})
            .for_each( |node| node.set_up( parent ));
    }

    pub(crate) fn clear( &mut self ) {
        unsafe {
            let root = self.root.as_mut();
            root.head = None;
            root.tail = None;
            root.size = Size::default();
        }
    }

    /// Returns `true` if `Forest` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{tr, fr};
    /// let mut forest = fr();
    /// assert!( forest.has_no_child() );
    /// forest.push_back( tr(1) );
    /// assert!( !forest.has_no_child() );
    /// ```
    pub fn has_no_child( &self ) -> bool { self.root_().has_no_child() }

    /// Returns the number of child nodes in `Forest`.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Forest;
    /// let forest = Forest::<i32>::from_tuple(( 0, (1,2), (3,4) ));
    /// assert_eq!( forest.degree(), 3 );
    /// ```
    pub fn degree( &self ) -> usize { self.root_().degree() }

    /// Returns the number of all child nodes in `Forest`.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Forest;
    /// let forest = Forest::<i32>::from_tuple(( 0, (1,2), (3,4) ));
    /// assert_eq!( forest.node_count(), 5 );
    /// ```
    pub fn node_count( &self ) -> usize { self.root_().node_count() }

    /// Provides a forward iterator over child `Node`s.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{tr, fr};
    ///
    /// let forest = fr::<i32>();
    /// assert_eq!( forest.iter().next(), None );
    ///
    /// let forest = -tr(1)-tr(2);
    /// let mut iter = forest.iter();
    /// assert_eq!( iter.next(), Some( tr(1).root() ));
    /// assert_eq!( iter.next(), Some( tr(2).root() ));
    /// assert_eq!( iter.next(), None );
    /// ```
    pub fn iter<'a, 's:'a>( &'s self ) -> Iter<'a,T> { self.root_().iter() }

    /// Provides a forward iterator over child `Node`s with mutable references.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Forest;
    ///
    /// let mut forest = Forest::<i32>::new();
    /// assert_eq!( forest.iter_mut().next(), None );
    ///
    /// let mut forest = Forest::<i32>::from_tuple(( 1, 2 ));
    /// forest.iter_mut().for_each( |mut child| { *child.data_mut() *= 10; });
    /// assert_eq!( forest.to_string(), "( 10 20 )" );
    /// ```
    pub fn iter_mut<'a, 's:'a>( &'s mut self ) -> IterMut<'a,T> { self.root_mut_().iter_mut() }

    /// Returns the first child of the forest,
    /// or `None` if it is empty.
    pub fn front( &self ) -> Option<&Node<T>> {
        self.root_().front()
    }

    /// Returns a mutable pointer to the first child of the forest,
    /// or `None` if it is empty.
    pub fn front_mut( &mut self ) -> Option<Pin<&mut Node<T>>> {
        self.root_mut_().front_mut()
    }

    pub fn back( &self ) -> Option<&Node<T>> {
        self.root_().back()
    }

    /// Returns a mutable pointer to the last child of the forest,
    /// or `None` if it is empty.
    pub fn back_mut( &mut self ) -> Option<Pin<&mut Node<T>>> {
        self.root_mut_().back_mut()
    }

    /// Add the tree as the first child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Tree, Forest};
    /// let mut forest = Forest::new();
    /// forest.push_front( Tree::new(1) );
    /// assert_eq!( forest.to_string(), "( 1 )" );
    /// forest.push_front( Tree::new(2) );
    /// assert_eq!( forest.to_string(), "( 2 1 )" );
    /// ```
    pub fn push_front( &mut self, tree: Tree<T> ) {
        self.root_mut_().push_front( tree );
    }

    /// Add the tree as the last child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Tree, Forest};
    /// let mut forest = Forest::new();
    /// forest.push_back( Tree::new(1) );
    /// assert_eq!( forest.to_string(), "( 1 )" );
    /// forest.push_back( Tree::new(2) );
    /// assert_eq!( forest.to_string(), "( 1 2 )" );
    /// ```
    pub fn push_back( &mut self, tree: Tree<T> ) {
        self.root_mut_().push_back( tree );
    }

    /// Remove and return the first child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Tree, Forest};
    /// let mut forest = Forest::new();
    /// forest.push_back( Tree::new(1) );
    /// forest.push_back( Tree::new(2) );
    /// assert_eq!( forest.to_string(), "( 1 2 )" );
    /// assert_eq!( forest.pop_front(), Some( Tree::new(1) ));
    /// assert_eq!( forest.to_string(), "( 2 )" );
    /// assert_eq!( forest.pop_front(), Some( Tree::new(2) ));
    /// assert_eq!( forest.to_string(), "()" );
    /// ```
    pub fn pop_front( &mut self ) -> Option<Tree<T>> {
        self.root_mut_().pop_front()
    }

    /// Remove and return the first child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Tree, Forest};
    /// let mut forest = Forest::new();
    /// forest.push_back( Tree::new(1) );
    /// forest.push_back( Tree::new(2) );
    /// assert_eq!( forest.to_string(), "( 1 2 )" );
    /// assert_eq!( forest.pop_back(), Some( Tree::new(2) ));
    /// assert_eq!( forest.to_string(), "( 1 )" );
    /// assert_eq!( forest.pop_back(), Some( Tree::new(1) ));
    /// assert_eq!( forest.to_string(), "()" );
    /// ```
    pub fn pop_back( &mut self ) -> Option<Tree<T>> {
        self.root_mut_().pop_back()
    }

    /// Add all the forest's trees at front of children list
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Tree, Forest};
    /// let mut forest = Forest::new();
    /// forest.push_back( Tree::new(1) );
    /// forest.push_back( Tree::new(2) );
    /// let mut forest2 = Forest::new();
    /// forest2.push_back( Tree::new(3) );
    /// forest2.push_back( Tree::new(4) );
    /// forest.prepend( forest2 );
    /// assert_eq!( forest.to_string(), "( 3 4 1 2 )" );
    /// ```
    pub fn prepend( &mut self, forest: Forest<T> ) {
        self.root_mut_().prepend( forest );
    }

    /// Add all the forest's trees at back of children list
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Tree, Forest};
    /// let mut forest = Forest::new();
    /// forest.push_back( Tree::new(1) );
    /// forest.push_back( Tree::new(2) );
    /// let mut forest2 = Forest::new();
    /// forest2.push_back( Tree::new(3) );
    /// forest2.push_back( Tree::new(4) );
    /// forest.append( forest2 );
    /// assert_eq!( forest.to_string(), "( 1 2 3 4 )" );
    /// ```
    pub fn append( &mut self, forest: Forest<T> ) {
        self.root_mut_().append( forest );
    }
}

impl<T> Default for Forest<T> { fn default() -> Self { Forest::new() }}

impl<T> Drop for Forest<T> {
    fn drop( &mut self ) {
        while let Some(_) = self.pop_front() {}
        heap::drop_node( self.root );
    }
}

impl<T:Clone> Clone for Forest<T> {
    fn clone( &self ) -> Self {
        self.root_().deep_clone_forest()
    }
}

impl_debug_display_for_forest!( Forest, iter() );
impl_order_relations_for_collection!( Forest, iter() );
impl_hash_for_forest!( Forest, iter() );

#[cfg( test )]
mod tests {
    use super::*;

    #[test] fn empty_piled_forest_from_tuple() {
        let tuple = ();
        let piled = Forest::<i32>::from_tuple( tuple );
        assert_eq!( piled.to_string(), "()" );
    }

    #[test] fn piled_forest_from_tuple() {
        let tuple = ( (2,3,4), (5,6,7) );
        let piled = Forest::<i32>::from_tuple( tuple );
        assert_eq!( piled.to_string(), "( 2( 3 4 ) 5( 6 7 ) )" );
    }
}

#[cfg( miri )]
mod miri_tests {
    #[test] fn has_no_child() {
        use crate::{fr, tr};

        let mut forest = fr();
        assert!( forest.has_no_child() );
        forest.push_back( tr(1) );
        assert!( !forest.has_no_child() );
    }

    #[test] fn degree() {
        use crate::Forest;

        let forest = Forest::<i32>::from_tuple(( 0, (1,2), (3,4) ));
        assert_eq!( forest.degree(), 3 );
    }

    #[test] fn node_count() {
        use crate::Forest;

        let forest = Forest::<i32>::from_tuple(( 0, (1,2), (3,4) ));
        assert_eq!( forest.node_count(), 5 );
    }

    #[test] fn iter() {
        use crate::{fr, tr};

        let forest = fr::<i32>();
        assert_eq!( forest.iter().next(), None );

        let forest = -tr(1)-tr(2);
        let mut iter = forest.iter();
        assert_eq!( iter.next(), Some( tr(1).root() ));
        assert_eq!( iter.next(), Some( tr(2).root() ));
        assert_eq!( iter.next(), None );
    }

    #[test] fn iter_mut() {
        use crate::Forest;

        let mut forest = Forest::<i32>::new();
        assert_eq!( forest.iter_mut().next(), None );

        let mut forest = Forest::<i32>::from_tuple(( 1, 2 ));
        forest.iter_mut().for_each( |mut child| { *child.data_mut() *= 10; });
        assert_eq!( forest.to_string(), "( 10 20 )" );
    }

    #[test] fn push_front() {
        use crate::{Forest, Tree};

        let mut forest = Forest::new();
        forest.push_front( Tree::new(1) );
        assert_eq!( forest.to_string(), "( 1 )" );
        forest.push_front( Tree::new(2) );
        assert_eq!( forest.to_string(), "( 2 1 )" );
    }

    #[test] fn push_back() {
        use crate::{Forest, Tree};

        let mut forest = Forest::new();
        forest.push_back( Tree::new(1) );
        assert_eq!( forest.to_string(), "( 1 )" );
        forest.push_back( Tree::new(2) );
        assert_eq!( forest.to_string(), "( 1 2 )" );
    }

    #[test] fn pop_front() {
        use crate::{Forest, Tree};

        let mut forest = Forest::new();
        forest.push_back( Tree::new(1) );
        forest.push_back( Tree::new(2) );
        assert_eq!( forest.to_string(), "( 1 2 )" );
        assert_eq!( forest.pop_front(), Some( Tree::new(1) ));
        assert_eq!( forest.to_string(), "( 2 )" );
        assert_eq!( forest.pop_front(), Some( Tree::new(2) ));
        assert_eq!( forest.to_string(), "()" );
    }

    #[test] fn pop_back() {
        use crate::{Forest, Tree};

        let mut forest = Forest::new();
        forest.push_back( Tree::new(1) );
        forest.push_back( Tree::new(2) );
        assert_eq!( forest.to_string(), "( 1 2 )" );
        assert_eq!( forest.pop_back(), Some( Tree::new(2) ));
        assert_eq!( forest.to_string(), "( 1 )" );
        assert_eq!( forest.pop_back(), Some( Tree::new(1) ));
        assert_eq!( forest.to_string(), "()" );
    }

    #[test] fn prepend() {
        use crate::{Forest, Tree};

        let mut forest = Forest::new();
        forest.push_back( Tree::new(1) );
        forest.push_back( Tree::new(2) );
        let mut forest2 = Forest::new();
        forest2.push_back( Tree::new(3) );
        forest2.push_back( Tree::new(4) );
        forest.prepend( forest2 );
        assert_eq!( forest.to_string(), "( 3 4 1 2 )" );
    }

    #[test] fn append() {
        use crate::{Forest, Tree};

        let mut forest = Forest::new();
        forest.push_back( Tree::new(1) );
        forest.push_back( Tree::new(2) );
        let mut forest2 = Forest::new();
        forest2.push_back( Tree::new(3) );
        forest2.push_back( Tree::new(4) );
        forest.append( forest2 );
        assert_eq!( forest.to_string(), "( 1 2 3 4 )" );
    }

    #[test] fn from_tuple() {
        use crate::{Forest, tr};

        let forest = Forest::<i32>::from_tuple(( 0, (1,2), (3,4) ));
        assert_eq!( forest, -tr(0) -tr(1)/tr(2) -tr(3)/tr(4) );
        assert_eq!( forest.to_string(), "( 0 1( 2 ) 3( 4 ) )" );
    }
}
