//! Composed of `data` and a list of its child `Node`s.
//!
//! Can be converted to `RcNode`, which has shared ownership.

use crate::Size;

use crate::rust::*;

use super::{Forest, Iter, IterMut, NodeVec, Tree};

/// Data associated with `Node`.
#[derive( Debug, PartialEq, Eq, PartialOrd, Ord, Hash )]
pub(crate) enum Data<T> {
    None,
    ScatteredNone{          owner: NonNull<RefCell<Node<T>>> },
    Scattered    { data: T, owner: NonNull<RefCell<Node<T>>> },
    PiledNone    {          owner: NonNull<NodeVec<T>>       },
    Piled        { data: T, owner: NonNull<NodeVec<T>>       },
}

impl<T> Default for Data<T> {
    fn default() -> Self { Data::None }
}

impl<T> Data<T> {
    pub(crate) fn is_none( &self ) -> bool {
        match self {
            Data::None => true,
            _ => false,
        }
    }

    pub(crate) fn take( &mut self ) -> T {
        match self {
            Data::Scattered{ owner, .. } => {
                let data = Data::ScatteredNone{ owner: *owner };
                match mem::replace( self, data ) {
                    Data::Scattered{ data, .. } => data,
                    _ => unreachable!(),
                }
            },
            Data::Piled{ owner, .. } => {
                let data = Data::PiledNone{ owner: *owner };
                match mem::replace( self, data ) {
                    Data::Piled{ data, .. } => data,
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
    }

    pub(crate) fn replace( &mut self, other: Data<T> ) -> Data<T> {
        mem::replace( self, other )
    }

    pub(crate) fn into_inner( self ) -> T {
        match self {
            Data::Piled{ data, owner } => {
                NodeVec::decr_ref( owner );
                data
            },
            Data::Scattered{ data, owner } => {
                drop( unsafe{ Rc::from_raw( owner.as_ptr() )});
                data
            },
            _ => unreachable!(),
        }
    }
}

impl<T> AsRef<T> for Data<T> {
    fn as_ref( &self ) -> &T {
        match self {
            Data::Piled    { ref data, .. } => data,
            Data::Scattered{ ref data, .. } => data,
            _ => unreachable!(),
        }
    }
}

impl<T> AsMut<T> for Data<T> {
    fn as_mut( &mut self ) -> &mut T {
        match self {
            Data::Piled    { ref mut data, .. } => data,
            Data::Scattered{ ref mut data, .. } => data,
            _ => unreachable!(),
        }
    }
}

/// Composed of `data` and a list of its child `Node`s.
/// Size infomation tracked.
pub struct Node<T> {
    pub(crate) prev : Option<NonNull<Node<T>>>,
    pub(crate) next : Option<NonNull<Node<T>>>,
    pub(crate) head : Option<NonNull<Node<T>>>,
    pub(crate) tail : Option<NonNull<Node<T>>>,
    pub(crate) up   : Option<NonNull<Node<T>>>,
    pub(crate) size : Size,
    pub(crate) data : Data<T>,
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Node{ prev: None, next: None, head: None, tail: None, up: None, size: Size::default(), data: Data::default() }
    }
}

impl<T> Node<T> {
    /// Reference of its associated data.
    pub fn data( &self ) -> &T { self.data.as_ref() }

    /// Mutable reeference of its associated data.
    pub fn data_mut( &mut self ) -> &mut T { self.data.as_mut() }

    /// Returns `true` if `Node` has no child nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    /// let mut tree = Tree::new(0);
    /// let mut root = tree.root_mut();
    /// assert!( root.has_no_child() );
    /// root.push_back( Tree::new(1) );
    /// assert!( !root.has_no_child() );
    /// ```
    pub fn has_no_child( &self ) -> bool { self.head.is_none() }

    /// Returns the number of child nodes in `Node`.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    /// let mut tree = Tree::new(0);
    /// let mut root = tree.root_mut();
    /// assert_eq!( root.degree(), 0 );
    /// root.push_back( Tree::new(1) );
    /// assert_eq!( root.degree(), 1 );
    /// root.push_back( Tree::new(2) );
    /// assert_eq!( root.degree(), 2 );
    /// ```
    pub fn degree( &self ) -> usize { self.size.degree }

    /// Returns the number of all child nodes in `Node`, including itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let tree = Tree::<i32>::from_tuple(( 0, (1,2), (3,4) ));
    /// assert_eq!( tree.root().node_count(), 5 );
    /// ```
    pub fn node_count( &self ) -> usize {
        if self.is_forest() {
            self.size.descendants
        } else {
            self.size.descendants + 1
        }
    }

    /// Returns the parent node of this node,
    /// or None if it is the root node.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let tree = Tree::<i32>::from_tuple(( 0, 1, 2, 3 ));
    /// tree.root().iter().for_each( |child| {
    ///     assert_eq!( child.parent(), Some( tree.root()))
    /// });
    /// ```
    pub fn parent( &self ) -> Option<&Node<T>> {
        let mut node = self.non_null();
        unsafe {
            while let Some( parent ) = node.as_ref().up {
                if parent.as_ref().is_forest() {
                    node = parent;
                } else {
                    return Some( &*parent.as_ptr() );
                }
            }
        }
        None
    }

    /// Inserts sib tree before `self`.
    /// The newly inserted node will not be iterated over by the currently running iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::tr;
    ///
    /// let mut tree = tr(0) /tr(1)/tr(2);
    /// tree.iter_mut().for_each( |mut sub| sub.insert_prev_sib( tr(3) ));
    /// assert_eq!( tree.to_string(), "0( 3 1 3 2 )" );
    /// ```
    pub fn insert_prev_sib( &mut self, mut sib: Tree<T> ) {
        let mut up = self.up.unwrap();

        sib.root_mut_().up = Some( up );

        unsafe {
            if let Some( mut prev ) = self.prev {
                prev.as_mut().connect_next( sib.root_mut_() );
            } else {
                up.as_mut().head = Some( sib.root );
            }
            sib.root_mut_().connect_next( self );

            up.as_mut().inc_sizes( 1, sib.node_count() );
        }

        mem::forget( sib );
    }

    /// Inserts sib tree after `self`.
    /// The newly inserted node will not be iterated over by the currently running iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::tr;
    /// let mut tree = tr(0) /tr(1)/tr(2);
    /// tree.iter_mut().for_each( |mut sub| sub.insert_next_sib( tr(3) ));
    /// assert_eq!( tree.to_string(), "0( 1 3 2 3 )" );
    /// ```
    pub fn insert_next_sib( &mut self, mut sib: Tree<T> ) {
        let mut up = self.up.unwrap();

        sib.root_mut_().up = Some( up );

        unsafe {
            if let Some( mut next ) = self.next {
                sib.root_mut_().connect_next( next.as_mut() );
            } else {
                up.as_mut().tail = Some( sib.root );
            }
            self.connect_next( sib.root_mut_() );

            up.as_mut().inc_sizes( 1, sib.node_count() );
        }

        mem::forget( sib );
    }

    /// The subtree departs from its parent and becomes an indepent `Tree`.
    ///
    /// # Examples
    /// ```
    /// use trees::{tr, fr};
    ///
    /// let mut forest = fr()-tr(1)-tr(2)-tr(3);
    /// forest.iter_mut().for_each( |mut sub| { sub.detach(); });
    /// assert_eq!( forest, fr() );
    /// ```
    pub fn detach( &mut self ) -> Tree<T> {
        unsafe {
            let mut up = self.up.unwrap();

            match up.as_ref().size.degree {
                1 => {
                    up.as_mut().head = None;
                    up.as_mut().tail = None;
                }
                _ => if self.prev.is_none() {
                    let mut next = self.next.unwrap();
                    next.as_mut().prev = None;
                    up.as_mut().head = Some( next );
                    self.next = None;
                } else if self.next.is_none() {
                    let mut prev = self.prev.unwrap();
                    prev.as_mut().next = None;
                    up.as_mut().tail = Some( prev );
                    self.prev = None;
                } else {
                    self.prev.unwrap().as_mut().connect_next( self.next.unwrap().as_mut() );
                    self.prev = None;
                    self.next = None;
                },
            }

            up.as_mut().dec_sizes( 1, self.node_count() );
            self.up = None;
        }

        Tree{ root: self.non_null(), mark: PhantomData }
    }

    /// Provides a forward iterator over child `Node`s
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let mut tree = Tree::new(0);
    /// assert_eq!( tree.iter().next(), None );
    ///
    /// tree.push_back( Tree::new(1) );
    /// tree.push_back( Tree::new(2) );
    /// let mut iter = tree.root().iter();
    /// assert_eq!( iter.next(), Some( Tree::new(1).root() ));
    /// assert_eq!( iter.next(), Some( Tree::new(2).root() ));
    /// assert_eq!( iter.next(), None );
    /// ```
    pub fn iter<'a, 's:'a>( &'s self ) -> Iter<'a,T> {
        match self.head {
            Some( child ) => Iter::new( Some( child ), self.degree() ),
            None => Iter::new( None, 0 ),
        }
    }

    /// Provides a forward iterator over child `Node`s with mutable references.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let mut tree = Tree::<i32>::from_tuple(( 0, (1, 2, 3), ));
    /// tree.front_mut().unwrap()
    ///     .iter_mut()
    ///     .for_each( |mut child| *child.data_mut() *= 10 );
    /// assert_eq!( tree.to_string(), "0( 1( 20 30 ) )" );
    /// ```
    pub fn iter_mut<'a, 's:'a>( &'s mut self ) -> IterMut<'a,T> {
        match self.head {
            Some( child ) => IterMut::new( Some( child ), self.degree() ),
            None => IterMut::new( None, 0 ),
        }
    }

    /// Returns the first child of this node,
    /// or None if it has no child.
    pub fn front( &self ) -> Option<&Node<T>> {
        self.head.map( |head| unsafe{ &*head.as_ptr() })
    }

    /// Returns a mutable pointer to the first child of this node,
    /// or None if it has no child.
    pub fn front_mut( &mut self ) -> Option<Pin<&mut Node<T>>> {
        self.head.map( |head| unsafe{ Pin::new_unchecked( &mut *head.as_ptr() )})
    }

    /// Returns the last child of this node,
    /// or None if it has no child.
    pub fn back( &self ) -> Option<&Node<T>> {
        self.tail.map( |tail| unsafe{ &*tail.as_ptr() })
    }

    /// Returns a mutable pointer to the last child of this node,
    /// or None if it has no child.
    pub fn back_mut( &mut self ) -> Option<Pin<&mut Node<T>>> {
        self.tail.map( |tail| unsafe{ Pin::new_unchecked( &mut *tail.as_ptr() )})
    }

    /// Adds the tree as the first child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let mut tree = Tree::new(0);
    /// tree.root_mut().push_front( Tree::new(1) );
    /// assert_eq!( tree.to_string(), "0( 1 )" );
    /// tree.root_mut().push_front( Tree::new(2) );
    /// assert_eq!( tree.to_string(), "0( 2 1 )" );
    /// ```
    pub fn push_front( &mut self, mut tree: Tree<T> ) {
        tree.root_mut_().set_up( self );
        if self.has_no_child() {
            self.set_tail( tree.root() );
        } else {
            unsafe{ tree.root_mut_().connect_next( self.head.unwrap().as_mut() ); }
        }
        self.set_head( tree.root() );
        self.inc_sizes( 1, tree.root().node_count() );
        mem::forget( tree );
    }

    /// Adds the tree as the last child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let mut tree = Tree::new(0);
    /// tree.root_mut().push_back( Tree::new(1) );
    /// assert_eq!( tree.to_string(), "0( 1 )" );
    /// tree.root_mut().push_back( Tree::new(2) );
    /// assert_eq!( tree.to_string(), "0( 1 2 )" );
    /// ```
    pub fn push_back( &mut self, mut tree: Tree<T> ) {
        tree.root_mut_().set_up( self );
        if self.has_no_child() {
            self.set_head( tree.root() );
        } else {
            unsafe{ self.tail.unwrap().as_mut().connect_next( tree.root_mut_() ); }
        }
        self.set_tail( tree.root() );
        self.inc_sizes( 1, tree.root().node_count() );
        mem::forget( tree );
    }

    /// Removes and return the first child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let mut tree = Tree::<i32>::from_tuple(( 0, (1, 2, 3), ));
    /// assert_eq!( tree.to_string(), "0( 1( 2 3 ) )" );
    /// assert_eq!( tree.front_mut().unwrap().pop_front(), Some( Tree::new(2) ));
    /// assert_eq!( tree.to_string(), "0( 1( 3 ) )" );
    /// assert_eq!( tree.front_mut().unwrap().pop_front(), Some( Tree::new(3) ));
    /// assert_eq!( tree.to_string(), "0( 1 )" );
    /// ```
    pub fn pop_front( &mut self ) -> Option<Tree<T>> {
        match self.size.degree {
            0 => None,
            1 => unsafe {
                let head = self.head.unwrap();
                self.head = None;
                self.tail = None;
                self.dec_sizes( 1, head.as_ref().size.descendants+1 );
                Some( Tree::from_node( head ))
            },
            _ => unsafe {
                let mut head = self.head.unwrap();
                let mut new_head = head.as_ref().next.unwrap();
                new_head.as_mut().prev = None;
                head.as_mut().next = None;
                self.head = Some( new_head );
                self.dec_sizes( 1, head.as_ref().size.descendants+1 );
                Some( Tree::from_node( head ))
            },
        }
    }

    /// Removes and return the last child.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let mut tree = Tree::<i32>::from_tuple(( 0, (1, 2, 3), ));
    /// assert_eq!( tree.to_string(), "0( 1( 2 3 ) )" );
    /// assert_eq!( tree.front_mut().unwrap().pop_back(), Some( Tree::new(3) ));
    /// assert_eq!( tree.to_string(), "0( 1( 2 ) )" );
    /// assert_eq!( tree.front_mut().unwrap().pop_back(), Some( Tree::new(2) ));
    /// assert_eq!( tree.to_string(), "0( 1 )" );
    /// ```
    pub fn pop_back( &mut self ) -> Option<Tree<T>> {
        match self.size.degree {
            0 => None,
            1 => unsafe {
                let tail = self.tail.unwrap();
                self.head = None;
                self.tail = None;
                self.dec_sizes( 1, tail.as_ref().size.descendants+1 );
                Some( Tree::from_node( tail ))
            },
            _ => unsafe {
                let mut tail = self.tail.unwrap();
                let mut new_tail = tail.as_ref().prev.unwrap();
                new_tail.as_mut().next = None;
                tail.as_mut().prev = None;
                self.tail = Some( new_tail );
                self.dec_sizes( 1, tail.as_ref().size.descendants+1 );
                Some( Tree::from_node( tail ))
            },
        }
    }

    /// Adds all the forest's trees at front of children list.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Forest, Tree};
    /// let mut tree = Tree::new(0);
    /// tree.push_back( Tree::new(1) );
    /// tree.push_back( Tree::new(2) );
    /// let mut forest = Forest::new();
    /// forest.push_back( Tree::new(3) );
    /// forest.push_back( Tree::new(4) );
    /// tree.root_mut().prepend( forest );
    /// assert_eq!( tree.to_string(), "0( 3 4 1 2 )" );
    /// ```
    pub fn prepend( &mut self, mut forest: Forest<T> ) {
        if !forest.has_no_child() {
            forest.set_up( self );
            if self.has_no_child() {
                self.set_tail( forest.root_().back().unwrap() );
            } else {
                unsafe{ forest.root_mut_().tail.unwrap().as_mut().connect_next( self.head.unwrap().as_mut() ); }
            }
            self.set_head( forest.front().unwrap() );
            let size = forest.root_().size;
            self.inc_sizes( size.degree, size.descendants );
            forest.clear();
        }
    }

    /// Adds all the forest's trees at back of children list.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Forest, Tree};
    /// let mut tree = Tree::new(0);
    /// tree.root_mut().push_back( Tree::new(1) );
    /// tree.root_mut().push_back( Tree::new(2) );
    /// let mut forest = Forest::new();
    /// forest.push_back( Tree::new(3) );
    /// forest.push_back( Tree::new(4) );
    /// tree.root_mut().append( forest );
    /// assert_eq!( tree.to_string(), "0( 1 2 3 4 )" );
    /// ```
    pub fn append( &mut self, mut forest: Forest<T> ) {
        if !forest.has_no_child() {
            forest.set_up( self );
            if self.has_no_child() {
                self.set_head( forest.root_().front().unwrap() );
            } else {
                unsafe{ self.tail.unwrap().as_mut().connect_next( forest.root_mut_().head.unwrap().as_mut() ); }
            }
            self.set_tail( forest.back().unwrap() );
            let size = forest.root_().size;
            self.inc_sizes( size.degree, size.descendants );
            forest.clear();
        }
    }

    pub(crate) fn non_null( &self ) -> NonNull<Node<T>> {
        unsafe{ NonNull::new_unchecked( self as *const _ as *mut Node<T> )}
    }

    pub(crate) fn set_head( &mut self, child: &Node<T> ) {
        self.head = Some( child.non_null() );
    }

    pub(crate) fn set_tail( &mut self, child: &Node<T> ) {
        self.tail = Some( child.non_null() );
    }

    pub(crate) fn set_up( &mut self, up: &Node<T> ) {
        self.up = Some( up.non_null() );
    }

    pub(crate) fn connect_next( &mut self, next: &mut Node<T> ) {
        self.next = Some( next.non_null() );
        next.prev = Some( self.non_null() );
    }

    pub(crate) fn inc_sizes( &mut self, degree: usize, node_cnt: usize ) {
        self.size.degree += degree;
        self.size.descendants += node_cnt;
        let mut node = self.up;
        while let Some( mut pnode ) = node {
            unsafe {
                pnode.as_mut().size.descendants += node_cnt;
                node = pnode.as_ref().up;
            }
        }
    }

    pub(crate) fn dec_sizes( &mut self, degree: usize, node_cnt: usize ) {
        self.size.degree -= degree;
        self.size.descendants -= node_cnt;
        let mut node = self.up;
        while let Some( mut pnode ) = node {
            unsafe {
                pnode.as_mut().size.descendants -= node_cnt;
                node = pnode.as_ref().up;
            }
        }
    }

    pub(crate) fn is_forest( &self ) -> bool {
        match self.data {
            Data::PiledNone{ .. } => true,
            Data::ScatteredNone{ .. } => true,
            _ => false,
        }
    }
}

impl_debug_display_for_node!( Node, iter, data() );
impl_order_relations_for_node!( Node, iter, data() );
impl_hash_for_node!( Node, iter, data() );

#[cfg( miri )]
mod miri_tests {
    #[test] fn has_no_child() {
        use crate::Tree;

        let mut tree = Tree::new(0);
        let mut root = tree.root_mut();
        assert!( root.has_no_child() );
        root.push_back( Tree::new(1) );
        assert!( !root.has_no_child() );
    }

    #[test] fn degree() {
        use crate::Tree;

        let mut tree = Tree::new(0);
        let mut root = tree.root_mut();
        assert_eq!( root.degree(), 0 );
        root.push_back( Tree::new(1) );
        assert_eq!( root.degree(), 1 );
        root.push_back( Tree::new(2) );
        assert_eq!( root.degree(), 2 );
    }

    #[test] fn node_count() {
        use crate::Tree;

        let tree = Tree::<i32>::from_tuple(( 0, (1,2), (3,4) ));
        assert_eq!( tree.root().node_count(), 5 );
    }

    #[test] fn parent() {
        use crate::Tree;

        let tree = Tree::<i32>::from_tuple(( 0, 1, 2, 3 ));
        tree.root().iter().for_each( |child| {
            assert_eq!( child.parent(), Some( tree.root()))
        });
    }

    #[test] fn insert_prev_sib() {
        use crate::tr;

        let mut tree = tr(0) /tr(1)/tr(2);
        tree.iter_mut().for_each( |mut sub| sub.insert_prev_sib( tr(3) ));
        assert_eq!( tree.to_string(), "0( 3 1 3 2 )" );
    }

    #[test] fn insert_next_sib() {
        use crate::tr;

        let mut tree = tr(0) /tr(1)/tr(2);
        tree.iter_mut().for_each( |mut sub| sub.insert_next_sib( tr(3) ));
        assert_eq!( tree.to_string(), "0( 1 3 2 3 )" );
    }

    #[test] fn detach() {
        use crate::{fr, tr};

        let mut forest = -tr(1)-tr(2)-tr(3);
        forest.iter_mut().for_each( |mut sub| { sub.detach(); });
        assert_eq!( forest, fr() );
    }

    #[test] fn iter() {
        use crate::Tree;

        let mut tree = Tree::new(0);
        assert_eq!( tree.iter().next(), None );

        tree.push_back( Tree::new(1) );
        tree.push_back( Tree::new(2) );
        let mut iter = tree.root().iter();
        assert_eq!( iter.next(), Some( Tree::new(1).root() ));
        assert_eq!( iter.next(), Some( Tree::new(2).root() ));
        assert_eq!( iter.next(), None );
    }

    #[test] fn iter_mut() {
        use crate::Tree;

        let mut tree = Tree::<i32>::from_tuple(( 0, (1, 2, 3), ));
        tree.front_mut().unwrap()
            .iter_mut()
            .for_each( |mut child| *child.data_mut() *= 10 );
        assert_eq!( tree.to_string(), "0( 1( 20 30 ) )" );
    }

    #[test] fn push_front() {
        use crate::Tree;

        let mut tree = Tree::new(0);
        tree.root_mut().push_front( Tree::new(1) );
        assert_eq!( tree.to_string(), "0( 1 )" );
        tree.root_mut().push_front( Tree::new(2) );
        assert_eq!( tree.to_string(), "0( 2 1 )" );
    }

    #[test] fn push_back() {
        use crate::Tree;

        let mut tree = Tree::new(0);
        tree.root_mut().push_back( Tree::new(1) );
        assert_eq!( tree.to_string(), "0( 1 )" );
        tree.root_mut().push_back( Tree::new(2) );
        assert_eq!( tree.to_string(), "0( 1 2 )" );
    }

    #[test] fn pop_front() {
        use crate::Tree;

        let mut tree = Tree::<i32>::from_tuple(( 0, (1, 2, 3), ));
        assert_eq!( tree.to_string(), "0( 1( 2 3 ) )" );
        assert_eq!( tree.front_mut().unwrap().pop_front(), Some( Tree::new(2) ));
        assert_eq!( tree.to_string(), "0( 1( 3 ) )" );
        assert_eq!( tree.front_mut().unwrap().pop_front(), Some( Tree::new(3) ));
        assert_eq!( tree.to_string(), "0( 1 )" );
    }

    #[test] fn pop_back() {
        use crate::Tree;

        let mut tree = Tree::<i32>::from_tuple(( 0, (1, 2, 3), ));
        assert_eq!( tree.to_string(), "0( 1( 2 3 ) )" );
        assert_eq!( tree.front_mut().unwrap().pop_back(), Some( Tree::new(3) ));
        assert_eq!( tree.to_string(), "0( 1( 2 ) )" );
        assert_eq!( tree.front_mut().unwrap().pop_back(), Some( Tree::new(2) ));
        assert_eq!( tree.to_string(), "0( 1 )" );
    }

    #[test] fn prepend() {
        use crate::{Forest, Tree};

        let mut tree = Tree::new(0);
        tree.push_back( Tree::new(1) );
        tree.push_back( Tree::new(2) );
        let mut forest = Forest::new();
        forest.push_back( Tree::new(3) );
        forest.push_back( Tree::new(4) );
        tree.root_mut().prepend( forest );
        assert_eq!( tree.to_string(), "0( 3 4 1 2 )" );
    }

    #[test] fn append() {
        use crate::{Forest, Tree};

        let mut tree = Tree::new(0);
        tree.root_mut().push_back( Tree::new(1) );
        tree.root_mut().push_back( Tree::new(2) );
        let mut forest = Forest::new();
        forest.push_back( Tree::new(3) );
        forest.push_back( Tree::new(4) );
        tree.root_mut().append( forest );
        assert_eq!( tree.to_string(), "0( 1 2 3 4 )" );
    }
}
