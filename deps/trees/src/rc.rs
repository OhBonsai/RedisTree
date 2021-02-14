//! Reference-counting nodes.

use crate::rust::*;

use super::{Data, Forest, Node, NodeVec, IterRc, Tree};

// Replacement of std::rc::Rc to avoid heap allocations.
pub(crate) struct Shared<T> {
    pub(crate) data  : T,
    pub(crate) count : Cell<usize>, // Only strong count is required. Weak count is shared as NodeVec::ref_cnt.
}

impl<T> Shared<T> {
    pub(crate) fn new( data: T ) -> Self { Shared{ data, count: Cell::new(1), }}
}

impl<T> Deref for Shared<T> {
    type Target = T;

    fn deref( &self ) -> &T { &self.data }
}

pub(crate) trait RefCount {
    fn incr( &self );
    fn decr( &self ) -> usize;
}

impl RefCount for Cell<usize> {
    fn incr( &self ) {
        let count = self.get();
        if count == 0 || count == usize::MAX {
            panic!();
        } else {
            self.set( count + 1 );
        }
    }

    fn decr( &self ) -> usize {
        match self.get().checked_sub( 1 ) {
            Some( count ) => {
                self.set( count );
                count
            },
            None => panic!(),
        }
    }
}

/// Reference-counting node which stored scatteredly.
pub struct ScatteredRcNode<T>( Rc<RefCell<Node<T>>> );

/// Reference-counting node which stored contiguously.
pub struct PiledRcNode<T>( NonNull<NodeVec<T>>, usize );

/// Reference-counting node.
pub enum RcNode<T> {
    Scattered( ScatteredRcNode<T> ),
    Piled(         PiledRcNode<T> ),
}

impl<T> From<Tree<T>> for RcNode<T> {
    fn from( tree: Tree<T> ) -> Self {
        let rc = tree.root().into_rc();
        mem::forget( tree );
        rc
    }
}

impl<T> Node<T> {
    // increase ref count.
    pub(crate) fn rc( &self ) -> RcNode<T> {
        match &self.data {
            Data::Scattered{ data: _, owner } => {
                let rc = unsafe{ Rc::from_raw( owner.as_ptr() )};
                mem::forget( rc.clone() );
                RcNode::Scattered( ScatteredRcNode( rc ))
            },
            Data::Piled{ data: _, owner } => unsafe {
                let index = ( self as *const _ as usize - owner.as_ref().buf.as_ptr() as usize )
                    / mem::size_of::<Shared<RefCell<Node<T>>>>();
                let node_vec = owner.as_ref();
                node_vec.ref_cnt.incr();
                let node = node_vec.buf.get_unchecked( index );
                node.count.incr();
                RcNode::Piled( PiledRcNode( *owner, index ))
            },
            _ => unreachable!(),
        }
    }

    // do not increase ref count.
    pub(crate) fn into_rc( &self ) -> RcNode<T> {
        match &self.data {
            Data::Scattered{ data: _, owner } => {
                let rc = unsafe{ Rc::from_raw( owner.as_ptr() )};
                RcNode::Scattered( ScatteredRcNode( rc ))
            },
            Data::Piled{ data: _, owner } => unsafe {
                let index = ( self as *const _ as usize - owner.as_ref().buf.as_ptr() as usize )
                    / mem::size_of::<Shared<RefCell<Node<T>>>>();
                RcNode::Piled( PiledRcNode( *owner, index ))
            },
            _ => unreachable!(),
        }
    }
}

impl<T> Clone for RcNode<T> {
    fn clone( &self ) -> RcNode<T> {
        match self {
            RcNode::Scattered( ScatteredRcNode( rc )) => RcNode::Scattered( ScatteredRcNode( rc.clone() )),
            RcNode::Piled( PiledRcNode( node_vec, index )) => {
                unsafe {
                    let node = node_vec.as_ref().buf.get_unchecked( *index );
                    node.count.incr();
                }
                RcNode::Piled( PiledRcNode( node_vec.clone(), *index ))
            },
        }
    }
}

impl<T> Drop for RcNode<T> {
    fn drop( &mut self ) {
        let mut drop_piled = false;
        match self {
            RcNode::Scattered( ScatteredRcNode( rc )) => {
                if Rc::strong_count( &rc ) == 1 {
                    while let Some(_) = self.pop_front() {}
                }
            },
            RcNode::Piled( PiledRcNode( node_vec, index )) => unsafe {
                let node = node_vec.as_ref().buf.get_unchecked( *index );
                if node.count.decr() == 0 {
                    drop_piled = true;
                    while let Some(_) = self.pop_front() {}
                }
            },
        }
        match self {
            RcNode::Piled( PiledRcNode( node_vec, index )) => unsafe {
                if drop_piled {
                    let node = node_vec.as_mut().buf.get_unchecked_mut( *index );
                    ptr::drop_in_place( &mut node.data );
                }
                NodeVec::decr_ref( *node_vec );
            }
            _ => (),
        }
    }
}

impl<T> RcNode<T> {
    /// Checks if it is a root node.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trees::{RcNode, tr};
    ///
    /// let root = RcNode::from( tr(0)/tr(1) );
    /// assert!( root.is_root() );
    /// assert!( !root.front().unwrap().is_root() );
    /// ```
    pub fn is_root( &self ) -> bool { self.node_borrow().parent().is_none() }

    /// Dynamically borrows the node's data.
    pub fn data( &self ) -> Ref<T> { Ref::map( self.node_borrow(), |node| node.data() )}

    /// Mutably borrows the node's data.
    pub fn data_mut( &self ) -> RefMut<T> { RefMut::map( self.node_borrow_mut(), |node| node.data_mut() )}

    /// Obtains a node reference
    pub unsafe fn node( &self ) -> Ref<Node<T>> { self.node_borrow() }

    /// Obtains a mutable node reference
    pub unsafe fn node_mut( &self ) -> RefMut<Node<T>> { self.node_borrow_mut() }

    pub(crate) fn node_borrow( &self ) -> Ref<Node<T>> {
        match self {
            RcNode::Scattered( ScatteredRcNode( rc )) => {
                let borrowed = rc.deref().borrow();
                assert!( !borrowed.data.is_none() );
                unsafe{ transmute( borrowed )}
            },
            RcNode::Piled( PiledRcNode( node_vec, index )) => {
                let borrowed = unsafe{ node_vec.as_ref().buf.get_unchecked( *index ).deref().borrow() };
                assert!( !borrowed.data.is_none() );
                borrowed
            },
        }
    }

    pub(crate) fn node_borrow_mut( &self ) -> RefMut<Node<T>> {
        match self {
            RcNode::Scattered( ScatteredRcNode( rc )) => {
                let borrowed = rc.deref().borrow_mut();
                assert!( !borrowed.data.is_none() );
                unsafe{ transmute( borrowed )}
            },
            RcNode::Piled( PiledRcNode( node_vec, index )) => {
                let borrowed = unsafe{ node_vec.as_ref().buf.get_unchecked( *index ).deref().borrow_mut() };
                assert!( !borrowed.data.is_none() );
                borrowed
            },
        }
    }

    /// Returns `true` if this `Node` has no child node, otherwise `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trees::{RcNode, tr};
    ///
    /// let root = RcNode::from( tr(0)/tr(1) );
    /// assert!( !root.has_no_child() );
    /// assert!( root.front().unwrap().has_no_child() );
    /// ```
    pub fn has_no_child( &self ) -> bool { self.node_borrow().has_no_child() }

    /// Returns the number of subtrees.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{RcNode, Tree};
    ///
    /// let root = RcNode::from( Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), )));
    /// assert_eq!( root.degree(), 2 );
    /// ```
    pub fn degree( &self ) -> usize { self.node_borrow().degree() }

    /// Returns the number of all subnodes, including itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{RcNode, Tree};
    ///
    /// let root = RcNode::from( Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), )));
    /// assert_eq!( root.node_count(), 7 );
    /// ```
    pub fn node_count( &self ) -> usize { self.node_borrow().node_count() }

    /// Returns the first child of the tree,
    /// or None if it is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trees::{RcNode, tr};
    ///
    /// let root = RcNode::from( tr(0)/tr(1)/tr(2) );
    /// assert_eq!( root.front(), Some( RcNode::from( tr(1) )));
    /// ```
    pub fn front( &self ) -> Option<RcNode<T>> { self.node_borrow().front().map( |node| node.rc() )}

    /// Returns the last child of the tree,
    /// or None if it is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trees::{RcNode, tr};
    ///
    /// let root = RcNode::from( tr(0)/tr(1)/tr(2) );
    /// assert_eq!( root.back(), Some( RcNode::from( tr(2) )));
    /// ```
    pub fn back( &self ) -> Option<RcNode<T>> { self.node_borrow().back().map( |node| node.rc() )}

    /// Returns the parent node of this node,
    /// or None if it is the root node.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{RcNode, tr};
    ///
    /// let root = RcNode::from( tr(0)/tr(1) );
    /// assert_eq!( root.parent(), None );
    /// let tr_1 = root.front().unwrap();
    /// assert_eq!( tr_1.parent(), Some( root ));
    /// ```
    pub fn parent( &self ) -> Option<RcNode<T>> { self.node_borrow().parent().map( |node| node.rc() )}

    /// Adds the tree as the first child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{RcNode, Tree};
    ///
    /// let root = RcNode::from( Tree::new(0) );
    /// root.push_front( Tree::new(1) );
    /// assert_eq!( root.to_string(), "0( 1 )" );
    /// root.push_front( Tree::new(2) );
    /// assert_eq!( root.to_string(), "0( 2 1 )" );
    /// ```
    pub fn push_front( &self, tree: Tree<T> ) { self.node_borrow_mut().push_front( tree )}

    /// Adds the tree as the last child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{RcNode, Tree};
    ///
    /// let root = RcNode::from( Tree::new(0) );
    /// root.push_back( Tree::new(1) );
    /// assert_eq!( root.to_string(), "0( 1 )" );
    /// root.push_back( Tree::new(2) );
    /// assert_eq!( root.to_string(), "0( 1 2 )" );
    /// ```
    pub fn push_back( &self, tree: Tree<T> ) { self.node_borrow_mut().push_back( tree )}

    /// Removes and return the first child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{RcNode, Tree};
    ///
    /// let root = RcNode::from( Tree::<i32>::from_tuple(( 0,1,2 )));
    /// let front = root.pop_front().unwrap();
    /// assert_eq!( front, RcNode::from( Tree::new(1) ));
    /// assert_eq!( root.to_string(), "0( 2 )" );
    /// let front = root.pop_front().unwrap();
    /// assert_eq!( front, RcNode::from( Tree::new(2) ));
    /// assert_eq!( root.to_string(), "0" );
    /// ```
    pub fn pop_front( &self ) -> Option<RcNode<T>> {
        self.node_borrow_mut().pop_front().map( |tree| RcNode::from( tree ))
    }

    /// Removes and return the last child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{RcNode, Tree};
    ///
    /// let root = RcNode::from( Tree::<i32>::from_tuple(( 0,1,2 )));
    /// let back = root.pop_back().unwrap();
    /// assert_eq!( back, RcNode::from( Tree::new(2) ));
    /// assert_eq!( root.to_string(), "0( 1 )" );
    /// let back = root.pop_back().unwrap();
    /// assert_eq!( back, RcNode::from( Tree::new(1) ));
    /// assert_eq!( root.to_string(), "0" );
    /// ```
    pub fn pop_back( &self ) -> Option<RcNode<T>> { self.node_borrow_mut().pop_back().map( |tree| RcNode::from( tree ))}

    /// Adds all the forest's trees at front of children list.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Forest, RcNode, Tree, tr};
    ///
    /// let root = RcNode::from( Tree::<i32>::from_tuple(( 0, 1, 2 )));
    /// let forest = Forest::<i32>::from_tuple(( 3, 4 ));
    /// root.prepend( forest );
    /// assert_eq!( root.to_string(), "0( 3 4 1 2 )" );
    /// ```
    pub fn prepend( &self, forest: Forest<T> ) { self.node_borrow_mut().prepend( forest )}

    /// Adds all the forest's trees at back of children list.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Forest, RcNode, Tree, tr};
    ///
    /// let root = RcNode::from( Tree::<i32>::from_tuple(( 0, 1, 2 )));
    /// let forest = Forest::<i32>::from_tuple(( 3, 4 ));
    /// root.append( forest );
    /// assert_eq!( root.to_string(), "0( 1 2 3 4 )" );
    /// ```
    pub fn append( &self, forest: Forest<T> ) { self.node_borrow_mut().append( forest )}

    /// Inserts sib tree before `self`.
    /// The newly inserted node will not be iterated over by the currently running iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{RcNode, tr};
    ///
    /// let root = RcNode::from( tr(0) /tr(1)/tr(2) );
    /// for sub in root.iter_rc() { sub.insert_prev_sib( tr(3) ); }
    /// assert_eq!( root.to_string(), "0( 3 1 3 2 )" );
    /// ```
    pub fn insert_prev_sib( &self, sib: Tree<T> ) {
        self.node_borrow_mut().insert_prev_sib( sib );
    }

    /// Inserts sib tree after `self`.
    /// The newly inserted node will not be iterated over by the currently running iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{RcNode, tr};
    ///
    /// let root = RcNode::from( tr(0) /tr(1)/tr(2) );
    /// for sub in root.iter_rc() { sub.insert_next_sib( tr(3) ); }
    /// assert_eq!( root.to_string(), "0( 1 3 2 3 )" );
    /// ```
    pub fn insert_next_sib( &self, sib: Tree<T> ) {
        self.node_borrow_mut().insert_next_sib( sib );
    }

    /// The subtree departs from its parent and becomes an indepent `Tree`.
    ///
    /// # Examples
    /// ```
    /// use trees::{RcNode, tr};
    ///
    /// let root = RcNode::from( tr(0) /tr(1)/tr(2)/tr(3) );
    /// for sub in root.iter_rc() { sub.detach(); }
    /// assert!( root.has_no_child() );
    /// ```
    pub fn detach( &self ) {
        drop( RcNode::from( self.node_borrow_mut().detach() ));
    }

    /// Provides a forward iterator over child `Node`s, with shared ownership.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{RcNode, tr};
    ///
    /// let root = RcNode::from( tr(0) );
    /// assert_eq!( root.iter_rc().next(), None );
    ///
    /// let root = RcNode::from( tr(0) /tr(1)/tr(2) );
    /// let mut iter = root.iter_rc();
    /// assert_eq!( iter.next(), Some( RcNode::from( tr(1) )));
    /// assert_eq!( iter.next(), Some( RcNode::from( tr(2) )));
    /// assert_eq!( iter.next(), None );
    /// ```
    pub fn iter_rc( &self ) -> IterRc<T> {
        let node = self.node_borrow();
        if node.has_no_child() {
            IterRc::new( None, 0 )
        } else {
            IterRc::new( node.front().map( |front| front.non_null() ), node.degree() )
        }
    }

    /// Creates a new weak pointer to this node.
    pub fn downgrade( &self ) -> WeakNode<T> {
        match self {
            RcNode::Scattered( ScatteredRcNode( rc )) => WeakNode::Scattered( ScatteredWeakNode( Rc::downgrade( &rc ))),
            RcNode::Piled( PiledRcNode( node_vec, index )) => WeakNode::Piled( PiledWeakNode( *node_vec, *index )),
        }
    }

    /// Converts to a tree which disables reference-counting.
    ///
    /// # Panics
    ///
    /// Only root node could be converted, otherwise it panics.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{RcNode, Tree, tr};
    ///
    /// let root = RcNode::from( tr(0) /( tr(1)/tr(2) ));
    /// let tree = unsafe{ root.into_tree() };
    /// assert_eq!( tree, tr(0) /( tr(1)/tr(2) ));
    ///
    /// let root = RcNode::from( Tree::<i32>::from_tuple(( 0, (1, 2), )));
    /// let tree = unsafe{ root.into_tree() };
    /// assert_eq!( tree, tr(0) /( tr(1)/tr(2) ));
    /// ```
    pub unsafe fn into_tree( self ) -> Tree<T> {
        assert!( self.is_root() );
        let tree = Tree::from_node( match &self {
            RcNode::Scattered( ScatteredRcNode( rc )) => {
                NonNull::new_unchecked( &mut *rc.deref().borrow_mut() as *mut _ )
            },
            RcNode::Piled( PiledRcNode( node_vec, index )) => {
                node_vec.as_ref().non_null_node( *index )
            },
        });
        mem::forget( self );
        tree
    }
}

impl<T:Clone> RcNode<T> {
    /// Clones the node deeply and creates a new tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{RcNode, tr};
    ///
    /// let root = RcNode::from( tr(0) /( tr(1)/tr(2) ));
    /// let new_tree = root.front().unwrap().deep_clone();
    /// assert_eq!( new_tree, tr(1) /tr(2) );
    /// ```
    pub fn deep_clone( &self ) -> Tree<T> { self.node_borrow().deep_clone() }
}

impl<T> Extend<Tree<T>> for RcNode<T> {
    fn extend<I:IntoIterator<Item=Tree<T>>>( &mut self, iter: I ) {
        for child in iter.into_iter() {
            self.node_borrow_mut().push_back( child );
        }
    }
}

impl_debug_display_for_node!( RcNode, iter_rc, data().deref() );
impl_order_relations_for_node!( RcNode, iter_rc, data().deref() );
impl_hash_for_node!( RcNode, iter_rc, data().deref() );

/// Non-owning reference-counting node which stored scatteredly.
pub struct ScatteredWeakNode<T>( Weak<RefCell<Node<T>>> );

/// Non-owning reference-counting node which stored contiguously.
pub struct PiledWeakNode<T>( NonNull<NodeVec<T>>, usize );

/// Non-owning reference-counting node.
pub enum WeakNode<T> {
    Scattered( ScatteredWeakNode<T> ),
    Piled( PiledWeakNode<T> ),
}

impl<T> WeakNode<T> {
    /// Attempts to upgrade the `WeakNode` a `RcNode`, delaying dropping of the `Node` if successful.
    /// Returns None if the `Node` has since been dropped.
    pub fn upgrade( &self ) -> Option<RcNode<T>> {
        match self {
            WeakNode::Scattered( ScatteredWeakNode( weak )) =>
                weak.upgrade().map( |rc| RcNode::Scattered( ScatteredRcNode( rc ))),
            WeakNode::Piled( PiledWeakNode( node_vec, index )) => unsafe {
                let node = node_vec.as_ref().buf.get_unchecked( *index );
                if node.count.get() == 0 {
                    None
                } else {
                    node.count.incr();
                    Some( RcNode::Piled( PiledRcNode( *node_vec, *index )))
                }
            },
        }
    }
}

impl<T> Drop for WeakNode<T> {
    fn drop( &mut self ) {
        if let WeakNode::Piled( PiledWeakNode( node_vec, _ )) = self  {
            unsafe {
                if node_vec.as_ref().ref_cnt.decr() == 0 {
                    drop( Box::from_raw( node_vec.as_ptr() ));
                }
            }
        }
    }
}

#[cfg( test )]
mod tests {
    #[test]
    fn rc_works() {
        use super::super::{RcNode, tr};

        let rc_0 = RcNode::from( tr(0) /( tr(1)/tr(2) ));
        let rc_1 = rc_0.front().unwrap();
        let rc_2 = rc_1.front().unwrap();

        *rc_0.data_mut() = 3;
        *rc_1.data_mut() = 4;
        *rc_2.data_mut() = 5;
        assert_eq!( rc_0, RcNode::from( tr(3) /( tr(4)/tr(5) )));

        {
            let rc_4 = rc_0.pop_back().unwrap();
            assert_eq!( rc_4, RcNode::from( tr(4)/tr(5) ));
        }

        assert_eq!( *rc_1.data(), 4 );
        assert_eq!( *rc_2.data(), 5 );
    }
}

#[cfg( miri )]
mod miri_tests {
    #[test] fn is_root() {
        use crate::{RcNode, tr};

        let root = RcNode::from( tr(0)/tr(1) );
        assert!( root.is_root() );
        assert!( !root.front().unwrap().is_root() );
    }

    #[test] fn has_no_child() {
        use crate::{RcNode, tr};

        let root = RcNode::from( tr(0)/tr(1) );
        assert!( !root.has_no_child() );
        assert!( root.front().unwrap().has_no_child() );
    }

    #[test] fn degree() {
        use crate::{RcNode, Tree};

        let root = RcNode::from( Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), )));
        assert_eq!( root.degree(), 2 );
    }

    #[test] fn node_count() {
        use crate::{RcNode, Tree};

        let root = RcNode::from( Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), )));
        assert_eq!( root.node_count(), 7 );
    }

    #[test] fn front() {
        use crate::{RcNode, tr};

        let root = RcNode::from( tr(0)/tr(1)/tr(2) );
        assert_eq!( root.front(), Some( RcNode::from( tr(1) )));
    }

    #[test] fn back() {
        use crate::{RcNode, tr};

        let root = RcNode::from( tr(0)/tr(1)/tr(2) );
        assert_eq!( root.back(), Some( RcNode::from( tr(2) )));
    }

    #[test] fn parent() {
        use crate::{RcNode, tr};

        let root = RcNode::from( tr(0)/tr(1) );
        assert_eq!( root.parent(), None );
        let tr_1 = root.front().unwrap();
        assert_eq!( tr_1.parent(), Some( root ));
    }

    #[test] fn push_front() {
        use crate::{RcNode, Tree};

        let root = RcNode::from( Tree::new(0) );
        root.push_front( Tree::new(1) );
        assert_eq!( root.to_string(), "0( 1 )" );
        root.push_front( Tree::new(2) );
        assert_eq!( root.to_string(), "0( 2 1 )" );
    }

    #[test] fn push_back() {
        use crate::{RcNode, Tree};

        let root = RcNode::from( Tree::new(0) );
        root.push_back( Tree::new(1) );
        assert_eq!( root.to_string(), "0( 1 )" );
        root.push_back( Tree::new(2) );
        assert_eq!( root.to_string(), "0( 1 2 )" );
    }

    #[test] fn pop_front() {
        use crate::{RcNode, Tree};

        let root = RcNode::from( Tree::<i32>::from_tuple(( 0,1,2 )));
        let front = root.pop_front().unwrap();
        assert_eq!( front, RcNode::from( Tree::new(1) ));
        assert_eq!( root.to_string(), "0( 2 )" );
        let front = root.pop_front().unwrap();
        assert_eq!( front, RcNode::from( Tree::new(2) ));
        assert_eq!( root.to_string(), "0" );
    }

    #[test] fn pop_back() {
        use crate::{RcNode, Tree};

        let root = RcNode::from( Tree::<i32>::from_tuple(( 0,1,2 )));
        let back = root.pop_back().unwrap();
        assert_eq!( back, RcNode::from( Tree::new(2) ));
        assert_eq!( root.to_string(), "0( 1 )" );
        let back = root.pop_back().unwrap();
        assert_eq!( back, RcNode::from( Tree::new(1) ));
        assert_eq!( root.to_string(), "0" );
    }

    #[test] fn prepend() {
        use crate::{Forest, RcNode, Tree};

        let root = RcNode::from( Tree::<i32>::from_tuple(( 0, 1, 2 )));
        let forest = Forest::<i32>::from_tuple(( 3, 4 ));
        root.prepend( forest );
        assert_eq!( root.to_string(), "0( 3 4 1 2 )" );
    }

    #[test] fn append() {
        use crate::{Forest, RcNode, Tree};

        let root = RcNode::from( Tree::<i32>::from_tuple(( 0, 1, 2 )));
        let forest = Forest::<i32>::from_tuple(( 3, 4 ));
        root.append( forest );
        assert_eq!( root.to_string(), "0( 1 2 3 4 )" );
    }

    #[test] fn insert_prev_sib() {
        use crate::{RcNode, tr};

        let root = RcNode::from( tr(0) /tr(1)/tr(2) );
        for sub in root.iter_rc() { sub.insert_prev_sib( tr(3) ); }
        assert_eq!( root.to_string(), "0( 3 1 3 2 )" );
    }

    #[test] fn insert_next_sib() {
        use crate::{RcNode, tr};

        let root = RcNode::from( tr(0) /tr(1)/tr(2) );
        for sub in root.iter_rc() { sub.insert_next_sib( tr(3) ); }
        assert_eq!( root.to_string(), "0( 1 3 2 3 )" );
    }

    #[test] fn detach() {
        use crate::{RcNode, tr};

        let root = RcNode::from( tr(0) /tr(1)/tr(2)/tr(3) );
        for sub in root.iter_rc() { sub.detach(); }
        assert!( root.has_no_child() );
    }

    #[test] fn iter_rc() {
        use crate::{RcNode, tr};

        let root = RcNode::from( tr(0) );
        assert_eq!( root.iter_rc().next(), None );

        let root = RcNode::from( tr(0) /tr(1)/tr(2) );
        let mut iter = root.iter_rc();
        assert_eq!( iter.next(), Some( RcNode::from( tr(1) )));
        assert_eq!( iter.next(), Some( RcNode::from( tr(2) )));
        assert_eq!( iter.next(), None );
    }

    #[test] fn into_tree() {
        use crate::{RcNode, Tree, tr};

        let root = RcNode::from( tr(0) /( tr(1)/tr(2) ));
        let tree = unsafe{ root.into_tree() };
        assert_eq!( tree, tr(0) /( tr(1)/tr(2) ));

        let root = RcNode::from( Tree::<i32>::from_tuple(( 0, (1, 2), )));
        let tree = unsafe{ root.into_tree() };
        assert_eq!( tree, tr(0) /( tr(1)/tr(2) ));
    }

    #[test] fn deep_clone() {
        use crate::{RcNode, tr};

        let root = RcNode::from( tr(0) /( tr(1)/tr(2) ));
        let new_tree = root.front().unwrap().deep_clone();
        assert_eq!( new_tree, tr(1) /tr(2) );
    }
}
