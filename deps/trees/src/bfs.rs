//! Breadth first search.

use crate::rust::*;

use super::Size;

/// Visit a node in breadth first search.
#[derive(Debug, PartialEq, Eq)]
pub struct Visit<T> {
    pub data : T,
    pub size : Size,
}

/// Tree iterator for breadth first search.
pub struct BfsTree<Iter> {
    pub iter : Iter,
    pub size : Size,
}

impl<Item,Iter> BfsTree<Splitted<Iter>>
    where Iter: Iterator<Item=Item>
{
    pub fn from<Treelike>( treelike: Treelike, size: Size ) -> Self
        where Treelike: IntoIterator<Item=Item,IntoIter=Iter>
    {
        Self{ iter: Splitted::<Iter>::from( treelike ), size: size }
    }
}

impl<Iter> BfsTree<Iter> {
    pub fn wrap( self ) -> Bfs<Iter> { Bfs::Tree( self )}

    /// Takes a closure and creates another BfsTree which calls that closure on
    /// each `Visit::data`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trees::Tree;
    ///
    /// let tree = Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), ));
    /// assert_eq!( Tree::from( tree.bfs() ),
    ///     Tree::<&i32>::from_tuple(( &0, (&1,&2,&3), (&4,&5,&6), )));
    /// assert_eq!( Tree::from( tree.bfs().map( ToOwned::to_owned )),
    ///     Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), )));
    /// ```
    pub fn map<B,F,T>( self, mut f: F ) -> BfsTree<impl Iterator<Item=Visit<B>>>
        where Iter : Iterator<Item=Visit<T>>
            , F    : FnMut(T) -> B
    {
        BfsTree {
            iter: self.iter.map( move |visit| Visit{ data: f( visit.data ), size: visit.size }),
            size: self.size,
        }
    }
}

/// Forest iterator for breadth first search.
pub struct BfsForest<Iter> {
    pub iter : Iter,
    pub size : Size,
}

impl<Item,Iter> BfsForest<Splitted<Iter>>
    where Iter: Iterator<Item=Item>
{
    pub fn from<Treelike>( treelike: Treelike, size: Size ) -> Self
        where Treelike: IntoIterator<Item=Item,IntoIter=Iter>
    {
        Self{ iter: Splitted::<Iter>::from( treelike ), size: size }
    }
}

impl<Iter> BfsForest<Iter> {
    pub fn wrap( self ) -> Bfs<Iter> { Bfs::Forest( self )}

    /// Takes a closure and creates another BfsForest which calls that closure
    /// on each `Visit::data`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use trees::Forest;
    ///
    /// let forest = Forest::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), ));
    /// assert_eq!( Forest::from( forest.bfs() ),
    ///     Forest::<&i32>::from_tuple(( &0, (&1,&2,&3), (&4,&5,&6), )));
    /// assert_eq!( Forest::from( forest.bfs().map( ToOwned::to_owned )),
    ///     Forest::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), )));
    /// ```
    pub fn map<B,F,T>( self, mut f: F ) -> BfsForest<impl Iterator<Item=Visit<B>>>
        where Iter : Iterator<Item=Visit<T>>
            , F    : FnMut(T) -> B
    {
        BfsForest {
            iter: self.iter.map( move |visit| Visit{ data: f( visit.data ), size: visit.size }),
            size: self.size,
        }
    }
}

/// Bfs iterator of either tree or forest.
pub enum Bfs<Iter> {
    Tree(   BfsTree  <Iter> ),
    Forest( BfsForest<Iter> ),
}

impl<T,Iter> Bfs<Iter>
    where Iter: Iterator<Item=Visit<T>>
{
    /// Returns the iterator in breadth-first search.
    pub fn iter( self ) -> Iter {
        match self {
            Bfs::Tree(   tree   ) => tree.iter,
            Bfs::Forest( forest ) => forest.iter,
        }
    }

    /// Returns the iterator and size infomation.
    pub fn iter_and_size( self ) -> ( Iter, Size ) {
        match self {
            Bfs::Tree(   tree   ) => (tree.iter,   tree.size),
            Bfs::Forest( forest ) => (forest.iter, forest.size),
        }
    }

    /// Returns the iterator which iterates the tree nodes in breadth-first
    /// search, or `None` if it is created by some `Forest`.
    pub fn tree_iter( self ) -> Option<Iter> {
        match self {
            Bfs::Tree( tree ) => Some( tree.iter ),
            _ => None,
        }
    }

    /// Returns the iterator which iterates the forest nodes in breadth-first
    /// search, or `None` if it is created by some `Tree`.
    pub fn forest_iter( self ) -> Option<Iter> {
        match self {
            Bfs::Forest( forest ) => Some( forest.iter ),
            _ => None,
        }
    }
}

/// Split tree node into data item and children iter.
pub trait Split {
    type Item;
    type Iter: ExactSizeIterator;

    fn split( self ) -> (Self::Item, Self::Iter, usize);
}

/// An iterator in breadth-first manner.
#[derive( Debug )]
pub struct Splitted<Iter> {
    pub(crate) iters : VecDeque<Iter>,
}

impl<Treelike,Item,Iter> From<Treelike> for Splitted<Iter>
    where Treelike : IntoIterator<Item=Item,IntoIter=Iter>
        ,     Iter : Iterator<Item=Item>
{
    fn from( treelike: Treelike ) -> Self {
        let mut iters = VecDeque::new();
        iters.push_back( treelike.into_iter() );
        Splitted{ iters }
    }
}

impl<T,Item,Iter> Iterator for Splitted<Iter>
    where Iter : ExactSizeIterator<Item=Item>
        , Item : Split<Iter=Iter,Item=T>
{
    type Item = Visit<T>;

    fn next( &mut self ) -> Option<Self::Item> {
        loop {
            let next_item =
                if let Some( ref mut iter ) = self.iters.front_mut() {
                    iter.next()
                } else {
                    return None;
                };
            if let Some( item ) = next_item {
                let (data, iter, descendants) = item.split();
                let degree = iter.len();
                self.iters.push_back( iter );
                return Some( Visit{ data, size: Size{ degree, descendants }});
            } else {
                self.iters.pop_front();
            }
        }
    }
}

#[cfg( miri )]
mod miri_tests {
    mod bfs_tree {
        #[test] fn map() {
            use crate::Tree;

            let tree = Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), ));
            assert_eq!( Tree::from( tree.bfs() ),
                Tree::<&i32>::from_tuple(( &0, (&1,&2,&3), (&4,&5,&6), )));
            assert_eq!( Tree::from( tree.bfs().map( ToOwned::to_owned )),
                Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), )));
        }
    }

    mod bfs_forest {
        #[test] fn map() {
            use crate::Forest;

            let forest = Forest::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), ));
            assert_eq!( Forest::from( forest.bfs() ),
                Forest::<&i32>::from_tuple(( &0, (&1,&2,&3), (&4,&5,&6), )));
            assert_eq!( Forest::from( forest.bfs().map( ToOwned::to_owned )),
                Forest::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), )));
        }
    }
}
