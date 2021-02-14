//! Forest's owning iterator.

use crate::rust::*;

use super::{Forest, Iter, IterMut, Node, Tree};

/// Forest's owning iterator.
pub struct IntoIter<T> {
    pub(crate) forest : Forest<T>,
    pub(crate) marker : PhantomData<Tree<T>>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = Tree<T>;

    fn next( &mut self ) -> Option<Tree<T>> {
        self.forest.pop_front()
    }

    fn size_hint( &self ) -> (usize, Option<usize>) {
        let degree = self.forest.degree();
        (degree, Some( degree ))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> Drop for IntoIter<T> {
    fn drop( &mut self ) {
        for _ in self.by_ref() {}
    }
}

impl<T> IntoIterator for Tree<T> {
    type Item = Tree<T>;
    type IntoIter = IntoIter<T>;

    fn into_iter( self ) -> IntoIter<T> {
        let mut forest = Forest::<T>::new();
        forest.push_back( self );
        IntoIter{ forest, marker: PhantomData }
    }
}

impl<T> IntoIterator for Forest<T> {
    type Item = Tree<T>;
    type IntoIter = IntoIter<T>;

    fn into_iter( self ) -> IntoIter<T> { IntoIter{ forest: self, marker: PhantomData }}
}

impl<'a, T:'a> IntoIterator for &'a Node<T> {
    type Item = Self;
    type IntoIter = Iter<'a,T>;

    fn into_iter( self ) -> Self::IntoIter {
        Iter::once( Some( self.non_null() ))
    }
}

impl<'a, T:'a> IntoIterator for Pin<&'a mut Node<T>> {
    type Item = Self;
    type IntoIter = IterMut<'a,T>;

    fn into_iter( self ) -> Self::IntoIter {
        IterMut::once( Some( self.non_null() ))
    }
}
