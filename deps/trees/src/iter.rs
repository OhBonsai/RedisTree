//! Iterators of `Tree`/`Forest`, returned by `iter()` or `iter_mut()`.

use crate::rust::*;

use super::Node;

#[derive( Debug )]
pub(crate) struct UncountedRawIter<T> {
    curr  : Option<NonNull<Node<T>>>,
}

impl<T> UncountedRawIter<T> {
    pub(crate) fn new( curr: Option<NonNull<Node<T>>> ) -> UncountedRawIter<T> {
        UncountedRawIter{ curr }
    }
}

impl<T> Copy for UncountedRawIter<T> {}

impl<T> Clone for UncountedRawIter<T> {
    fn clone( &self ) -> Self {
        UncountedRawIter{ curr: self.curr.clone() }
    }
}

impl<T> Iterator for UncountedRawIter<T> {
    type Item = NonNull<Node<T>>;

    fn next( &mut self ) -> Option<Self::Item> {
        self.curr.map( |curr| unsafe {
            let item = curr;
            self.curr = curr.as_ref().next;
            item
        })
    }
}

#[derive( Debug )]
pub(crate) struct CountedRawIter<T> {
    iter : UncountedRawIter<T>,
    len  : usize,
}

impl<T> CountedRawIter<T> {
    pub(crate) fn new( curr: Option<NonNull<Node<T>>>, len: usize ) -> CountedRawIter<T> {
        CountedRawIter {
            iter : UncountedRawIter::new( curr ),
            len  ,
        }
    }

    pub(crate) fn once( curr: Option<NonNull<Node<T>>> ) -> CountedRawIter<T> {
        CountedRawIter::<T>::new( curr, 1 )
    }
}

impl<T> Copy for CountedRawIter<T> {}

impl<T> Clone for CountedRawIter<T> {
    fn clone( &self ) -> Self {
        CountedRawIter{ iter: self.iter.clone(), len: self.len }
    }
}

impl<T> Iterator for CountedRawIter<T> {
    type Item = NonNull<Node<T>>;

    fn next( &mut self ) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        } else {
            self.len -= 1;
            return self.iter.next();
        }
    }

    fn size_hint( &self ) -> ( usize, Option<usize> ) {
        (self.len, Some( self.len ))
    }
}

/// An iterator over the child `Node`s of `Tree`, `Node` or `Forest`.
///
/// This `struct` is created by [`Node::iter`] and [`Forest::iter`].
/// See its document for more.
///
/// [`Node::iter`]: ../node/struct.Node.html#method.iter
/// [`Forest::iter`]: ../forest/struct.Forest.html#method.iter
#[derive( Debug )]
pub struct Iter<'a, T> {
    iter : CountedRawIter<T>,
    mark : PhantomData<&'a Node<T>>,
}

impl<'a,T:'a> Iter<'a,T> {
    pub(crate) fn new( curr: Option<NonNull<Node<T>>>, len: usize ) -> Iter<'a,T> {
        Iter{ iter: CountedRawIter::<T>::new( curr, len ), mark: PhantomData }
    }

    pub(crate) fn once( curr: Option<NonNull<Node<T>>> ) -> Iter<'a,T> {
        Iter{ iter: CountedRawIter::<T>::once( curr ), mark: PhantomData }
    }
}

impl<'a,T:'a> Iterator for Iter<'a,T> {
    type Item = &'a Node<T>;

    fn next( &mut self ) -> Option<Self::Item> {
        self.iter.next().map( |node| unsafe{ &*node.as_ptr() })
    }

    fn size_hint( &self ) -> ( usize, Option<usize> ) { self.iter.size_hint() }
}

impl<'a,T> ExactSizeIterator for Iter<'a, T> {}
impl<'a,T> FusedIterator for Iter<'a, T> {}

/// A mutable iterator over the child `Node`s of `Tree`, `Node` or `Forest`.
///
/// This `struct` is created by [`Node::iter_mut`] and [`Forest::iter_mut`].
/// See its document for more.
///
/// [`Node::iter_mut`]: ../node/struct.Node.html#method.iter_mut
/// [`Forest::iter_mut`]: ../forest/struct.Forest.html#method.iter_mut
#[derive( Debug )]
pub struct IterMut<'a, T> {
    iter : CountedRawIter<T>,
    mark : PhantomData<&'a mut Node<T>>,
}

impl<'a,T:'a> IterMut<'a,T> {
    pub(crate) fn new( curr: Option<NonNull<Node<T>>>, len: usize ) -> IterMut<'a,T> {
        IterMut{ iter: CountedRawIter::<T>::new( curr, len ), mark: PhantomData }
    }

    pub(crate) fn once( curr: Option<NonNull<Node<T>>> ) -> IterMut<'a,T> {
        IterMut{ iter: CountedRawIter::<T>::once( curr ),  mark: PhantomData }
    }
}

impl<'a,T:'a> Iterator for IterMut<'a,T> {
    type Item = Pin<&'a mut Node<T>>;

    fn next( &mut self ) -> Option<Self::Item> {
        self.iter.next().map( |node| unsafe{ Pin::new_unchecked( &mut *node.as_ptr() )})
    }

    fn size_hint( &self ) -> ( usize, Option<usize> ) { self.iter.size_hint() }
}

impl<'a,T> ExactSizeIterator for IterMut<'a, T> {}
impl<'a,T> FusedIterator for IterMut<'a, T> {}
