//! Iterators of `RcNode`, returned by `iter_rc()`.

use crate::rust::*;

use super::{CountedRawIter, Node, RcNode};

/// An iterator over the child `Node`s of `RcNode` with shared ownership.
///
/// This `struct` is created by [`RcNode::iter_rc`].
/// See its document for more.
///
/// [`RcNode::iter_rc`]: ../rc/enum.RcNode.html#method.iter_rc
pub struct IterRc<T> {
    iter : CountedRawIter<T>,
    mark : PhantomData<RcNode<T>>,
}

impl<T> Iterator for IterRc<T> {
    type Item = RcNode<T>;

    fn next( &mut self ) -> Option<RcNode<T>> {
        self.iter.next().map( |node| unsafe{ node.as_ref().rc() })
    }

    fn size_hint( &self ) -> ( usize, Option<usize> ) { self.iter.size_hint() }
}

impl<T> IterRc<T> {
    pub(crate) fn new( curr: Option<NonNull<Node<T>>>, len: usize ) -> Self {
        IterRc {
            iter: CountedRawIter::new( curr, len ),
            mark: PhantomData,
        }
    }
}

impl<T> Clone for IterRc<T> {
    fn clone( &self ) -> Self {
        IterRc { ..*self }
    }
}
