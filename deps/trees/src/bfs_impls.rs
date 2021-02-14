use crate::Size;

use crate::bfs::{BfsForest, BfsTree, Split, Splitted, Visit};

use crate::rust::*;

use super::{Data, Forest, IntoIter, Iter, IterMut, Node, NodeVec, Tree};

impl<T> Node<T> {
    /// Clones the node deeply and creates a new tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let tree = Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), (7,8,9), ));
    /// assert_eq!( tree.iter().nth(1).unwrap().deep_clone(),
    ///     Tree::from_tuple(( 4,5,6 )));
    /// ```
    pub fn deep_clone( &self ) -> Tree<T>
        where T: Clone
    {
        let (iter,size) = self.bfs().wrap().iter_and_size();
        let bfs_tree = BfsTree {
            iter: iter.map( |visit| Visit{ data: visit.data.clone(), size: visit.size }),
            size,
        };

        Tree::from( bfs_tree )
    }

    /// Clones the node's descendant nodes as a forest.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{Tree,Forest};
    ///
    /// let tree = Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), (7,8,9), ));
    /// assert_eq!( tree.iter().nth(1).unwrap().deep_clone_forest(),
    ///     Forest::from_tuple(( 5,6 )));
    /// ```
    pub fn deep_clone_forest( &self ) -> Forest<T>
        where T: Clone
    {
        let (iter,size) = self.bfs_children().wrap().iter_and_size();
        let bfs_forest = BfsForest {
            iter: iter.map( |visit| Visit{ data: visit.data.clone(), size: visit.size }),
            size,
        };

        Forest::from( bfs_forest )
    }

    /// Provides a forward iterator in a breadth-first manner, which iterates over all its descendants.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let tree = Tree::from_tuple(( 0, (1,2,3), (4,5,6), ));
    /// let visits = tree.root().bfs_children().iter
    ///     .map( |visit| (*visit.data, visit.size.degree, visit.size.descendants ))
    ///     .collect::<Vec<_>>();
    /// assert_eq!( visits, vec![ (1, 2, 2), (4, 2, 2), (2, 0, 0), (3, 0, 0), (5, 0, 0), (6, 0, 0), ]);
    /// ```
    pub fn bfs_children( &self ) -> BfsForest<Splitted<Iter<T>>> { BfsForest::from( self.iter(), self.size )}

    /// Provides a forward iterator with mutable references in a breadth-first manner, which iterates over all its descendants.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{tr, Tree};
    ///
    /// let mut tree = Tree::from_tuple(( 0, (1,2,3), (4,5,6), ));
    /// let mut root = tree.root_mut();
    /// root.bfs_children_mut().iter
    ///     .zip( 1.. )
    ///     .for_each( |(visit,nth)| *visit.data += 10 * nth );
    /// assert_eq!( tree, Tree::<i32>::from_tuple(( 0, (11,32,43), (24,55,66), )));
    /// ```
    pub fn bfs_children_mut( &mut self ) -> BfsForest<Splitted<IterMut<T>>> {
        let size = self.size;
        BfsForest::from( self.iter_mut(), size )
    }

    /// Provides a forward iterator in a breadth-first manner.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Tree;
    ///
    /// let tree = Tree::from_tuple(( 0, (1,2,3), (4,5,6), ));
    /// let visits = tree.root().bfs().iter
    ///     .map( |visit| (*visit.data, visit.size.degree, visit.size.descendants ))
    ///     .collect::<Vec<_>>();
    /// assert_eq!( visits, vec![ (0, 2, 6), (1, 2, 2), (4, 2, 2), (2, 0, 0), (3, 0, 0), (5, 0, 0), (6, 0, 0), ]);
    /// ```
    pub fn bfs( &self ) -> BfsTree<Splitted<Iter<T>>> {
        BfsTree::from( self.into_iter(), Size{ degree: 1, descendants: self.size.descendants })
    }

    /// Provides a forward iterator with mutable references in a breadth-first manner.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{tr, Tree};
    ///
    /// let mut tree = Tree::from_tuple(( 0, (1,2,3), (4,5,6), ));
    /// let mut root = tree.root_mut();
    /// root.bfs_mut().iter
    ///     .zip( 1.. )
    ///     .for_each( |(visit,nth)| *visit.data += 10 * nth );
    /// assert_eq!( tree, Tree::<i32>::from_tuple(( 10, (21,42,53), (34,65,76), )));
    /// ```
    pub fn bfs_mut( &mut self ) -> BfsTree<Splitted<IterMut<T>>> {
        let size = Size{ degree: 1, descendants: self.size.descendants };
        BfsTree::from( unsafe{ Pin::new_unchecked( self )}, size )
    }
}

impl<'a, T:'a> Split for &'a Node<T> {
    type Item = &'a T;
    type Iter = Iter<'a,T>;

    fn split( self ) -> (Self::Item, Self::Iter, usize) {
        (self.data(), self.iter(), self.size.descendants)
    }
}

impl<'a, T:'a> Split for Pin<&'a mut Node<T>> {
    type Item = &'a mut T;
    type Iter = IterMut<'a,T>;

    fn split( self ) -> (Self::Item, Self::Iter, usize) {
        let descendants = self.size.descendants;
        unsafe {
            let node_mut = self.get_unchecked_mut() as *mut Node<T>;
            let data = (*node_mut).data_mut() as *mut T;
            let iter = (*node_mut).iter_mut();
            (&mut *data, iter, descendants)
        }
    }
}

impl<T> Forest<T> {
    /// Provides a forward iterator in a breadth-first manner.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Forest;
    ///
    /// let forest = Forest::from_tuple(( (1,2,3), (4,5,6), ));
    /// let visits = forest.bfs().iter
    ///     .map( |visit| (*visit.data, visit.size.degree, visit.size.descendants ))
    ///     .collect::<Vec<_>>();
    /// assert_eq!( visits, vec![ (1, 2, 2), (4, 2, 2), (2, 0, 0), (3, 0, 0), (5, 0, 0), (6, 0, 0), ]);
    /// ```
    pub fn bfs( &self ) -> BfsForest<Splitted<Iter<T>>> { BfsForest::from( self.iter(), self.root_().size )}

    /// Provides a forward iterator with mutable references in a breadth-first manner.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::Forest;
    ///
    /// let mut forest = Forest::<i32>::from_tuple(( (1,2,3), (4,5,6), ));
    /// forest.bfs_mut().iter
    ///     .zip( 0.. )
    ///     .for_each( |(visit,nth)| *visit.data += 10 * nth );
    /// assert_eq!( forest, Forest::from_tuple(( (1,(22,),(33,)), (14,(45,),(56,)), )));
    /// ```
    pub fn bfs_mut( &mut self ) -> BfsForest<Splitted<IterMut<T>>> {
        let size = self.root_().size;
        BfsForest::from( self.iter_mut(), size )
    }

    /// Provides a forward iterator with owned data in a breadth-first manner.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{bfs,Size};
    /// use trees::Forest;
    ///
    /// let forest = Forest::<i32>::new();
    /// let visits = forest.into_bfs().iter.collect::<Vec<_>>();
    /// assert!( visits.is_empty() );
    ///
    /// let forest = Forest::from_tuple(( (1,2,3), (4,5,6), ));
    /// let visits = forest.into_bfs().iter.collect::<Vec<_>>();
    /// assert_eq!( visits, vec![
    ///     bfs::Visit{ data: 1, size: Size{ degree: 2, descendants: 2 }},
    ///     bfs::Visit{ data: 4, size: Size{ degree: 2, descendants: 2 }},
    ///     bfs::Visit{ data: 2, size: Size{ degree: 0, descendants: 0 }},
    ///     bfs::Visit{ data: 3, size: Size{ degree: 0, descendants: 0 }},
    ///     bfs::Visit{ data: 5, size: Size{ degree: 0, descendants: 0 }},
    ///     bfs::Visit{ data: 6, size: Size{ degree: 0, descendants: 0 }},
    /// ]);
    /// ```
    pub fn into_bfs( self: Forest<T> ) -> BfsForest<Splitted<IntoIter<T>>> {
        let size = self.root_().size;
        BfsForest::from( self.into_iter(), size )
    }
}

impl<T> Tree<T> {
    /// Provides a forward iterator with mutable references in a breadth-first manner, which iterates over all its descendants.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{tr, Tree};
    ///
    /// let mut tree = Tree::from_tuple(( 0, (1,2,3), (4,5,6), ));
    /// tree.bfs_children_mut().iter
    ///     .zip( 1.. )
    ///     .for_each( |(visit,nth)| *visit.data += 10 * nth );
    /// assert_eq!( tree, Tree::<i32>::from_tuple(( 0, (11,32,43), (24,55,66), )));
    /// ```
    pub fn bfs_children_mut( &mut self ) -> BfsForest<Splitted<IterMut<T>>> { self.root_mut_().bfs_children_mut() }

    /// Provides a forward iterator with mutable references in a breadth-first manner.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{tr, Tree};
    ///
    /// let mut tree = Tree::from_tuple(( 0, (1,2,3), (4,5,6), ));
    /// tree.bfs_mut().iter
    ///     .zip( 1.. )
    ///     .for_each( |(visit,nth)| *visit.data += 10 * nth );
    /// assert_eq!( tree, Tree::<i32>::from_tuple(( 10, (21,42,53), (34,65,76), )));
    /// ```
    pub fn bfs_mut( &mut self ) -> BfsTree<Splitted<IterMut<T>>> { self.root_mut_().bfs_mut() }

    /// Provides a forward iterator with owned data in a breadth-first manner.
    ///
    /// # Examples
    ///
    /// ```
    /// use trees::{bfs,Size};
    /// use trees::Tree;
    ///
    /// let tree = Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6) ));
    /// let visits = tree.into_bfs().iter.collect::<Vec<_>>();
    /// assert_eq!( visits, vec![
    ///     bfs::Visit{ data: 0, size: Size{ degree: 2, descendants: 6 }},
    ///     bfs::Visit{ data: 1, size: Size{ degree: 2, descendants: 2 }},
    ///     bfs::Visit{ data: 4, size: Size{ degree: 2, descendants: 2 }},
    ///     bfs::Visit{ data: 2, size: Size{ degree: 0, descendants: 0 }},
    ///     bfs::Visit{ data: 3, size: Size{ degree: 0, descendants: 0 }},
    ///     bfs::Visit{ data: 5, size: Size{ degree: 0, descendants: 0 }},
    ///     bfs::Visit{ data: 6, size: Size{ degree: 0, descendants: 0 }},
    /// ]);
    /// ```
    pub fn into_bfs( self ) -> BfsTree<Splitted<IntoIter<T>>> {
        let size = Size{ degree: 1, descendants: self.root().size.descendants };
        BfsTree::from( self, size )
    }
}

impl<T,Iter> From<BfsTree<Iter>> for Tree<T>
    where Iter: Iterator<Item=Visit<T>>
{
    fn from( tree_iter: BfsTree<Iter> ) -> Self {
        let mut iter = tree_iter.wrap().iter();
        let visit_root = iter.next().unwrap();
        let (degree, node_cnt) = (visit_root.size.degree, visit_root.size.descendants+1);

        let mut node_vec = NodeVec::new_raw_non_null( node_cnt );

        unsafe {
            node_vec.as_mut().make_piled_node( None, 0, visit_root.data, visit_root.size );
        }

        let mut parent  = 0;
        let mut child   = 1;
        let mut remains = degree;

        while let Some( visit ) = iter.next() {
            unsafe {
                node_vec.as_mut().append_child( parent, child, visit.data, visit.size );
            }
            remains -= 1;
            while remains == 0 {
                parent += 1;
                unsafe {
                    remains = node_vec.as_mut().node( parent ).degree();
                }
                if parent == child { break; }
            }
            child += 1;
        }

        Tree::from_node( unsafe{ node_vec.as_ref().non_null_node(0) })
    }
}

impl<T> Split for Tree<T> {
    type Item = T;
    type Iter = IntoIter<T>;

    fn split( mut self ) -> (T, IntoIter<T>, usize) {
        let descendants = self.root().size.descendants;
        let iter = self.abandon().into_iter();
        let data = self.into_data();
        (data, iter, descendants)
    }
}

impl<T,Iter> From<BfsForest<Iter>> for Forest<T>
    where Iter: Iterator<Item=Visit<T>>
{
    fn from( forest_iter: BfsForest<Iter> ) -> Self {
        let (mut iter, size) = forest_iter.wrap().iter_and_size();
        let (degree, node_cnt) = (size.degree, size.descendants+1);

        let mut node_vec = NodeVec::new_raw_non_null( node_cnt );

        unsafe {
            let fake_root = Data::PiledNone{ owner: node_vec };
            node_vec.as_mut().make_node( None, 0, fake_root, size );
        }

        let mut parent  = 0;
        let mut child   = 1;
        let mut remains = degree;

        while let Some( visit ) = iter.next() {
            unsafe {
                node_vec.as_mut().append_child( parent, child, visit.data, visit.size );
            }
            remains -= 1;
            while remains == 0 {
                parent += 1;
                unsafe {
                    remains = node_vec.as_mut().node( parent ).degree();
                }
                if parent == child { break; }
            }
            child += 1;
        }

        Forest::from_node( unsafe{ node_vec.as_ref().non_null_node(0) })
    }
}

#[cfg( test )]
mod tests {
    use super::*;
    use super::super::tr;

    #[test] fn piled_tree_from_bfs() {
        let linked = tr(0) /( tr(1)/tr(2)/tr(3) ) /( tr(4)/tr(5)/tr(6) );
        let piled = Tree::<i32>::from( linked.into_bfs() );
        assert_eq!( piled.to_string(), "0( 1( 2 3 ) 4( 5 6 ) )" );
    }

    #[test] fn piled_forest_from_bfs() {
        let linked = -( tr(1)/tr(2)/tr(3) ) -( tr(4)/tr(5)/tr(6) );
        let bfs = linked.into_bfs();
        let piled = Forest::<i32>::from( bfs );
        assert_eq!( piled.to_string(), "( 1( 2 3 ) 4( 5 6 ) )" );
    }

    #[test] fn piled_tree_from_bfs_1_1() {
        let linked = tr(1) /( tr(2) /tr(3) );
        let bfs = linked.into_bfs();
        let piled = Tree::<i32>::from( bfs );
        assert_eq!( piled.to_string(), "1( 2( 3 ) )" );
    }

    #[test] fn piled_forest_from_bfs_1_1() {
        let forest = -( tr(1) /tr(2) );
        let bfs = forest.into_bfs();
        let forest = Forest::<i32>::from( bfs );
        assert_eq!( forest.to_string(), "( 1( 2 ) )" );
    }
}

#[cfg( miri )]
mod miri_tests {
    mod node {
        #[test] fn deep_clone() {
            use crate::Tree;

            let tree = Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), (7,8,9), ));
            assert_eq!( tree.iter().nth(1).unwrap().deep_clone(), Tree::from_tuple(( 4,5,6 )));
        }

        #[test] fn deep_clone_forest() {
            use crate::{Forest, Tree};

            let tree = Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6), (7,8,9), ));
            assert_eq!( tree.iter().nth(1).unwrap().deep_clone_forest(), Forest::from_tuple(( 5,6 )));
        }

        #[test] fn bfs_children() {
            use crate::Tree;

            let tree = Tree::from_tuple(( 0, (1,2,3), (4,5,6), ));
            let visits = tree.root().bfs_children().iter.map( |visit| (*visit.data, visit.size.degree, visit.size.descendants )).collect::<Vec<_>>();
            assert_eq!( visits, vec![ (1, 2, 2), (4, 2, 2), (2, 0, 0), (3, 0, 0), (5, 0, 0), (6, 0, 0), ]);
        }

        #[test] fn bfs_children_mut() {
            use crate::Tree;

            let mut tree = Tree::from_tuple(( 0, (1,2,3), (4,5,6), ));
            let mut root = tree.root_mut();
            root.bfs_children_mut().iter.zip( 1.. ).for_each( |(visit,nth)| *visit.data += 10 * nth );
            assert_eq!( tree, Tree::<i32>::from_tuple(( 0, (11,32,43), (24,55,66), )));
        }

        #[test] fn bfs() {
            use crate::Tree;

            let tree = Tree::from_tuple(( 0, (1,2,3), (4,5,6), ));
            let visits = tree.root().bfs().iter.map( |visit| (*visit.data, visit.size.degree, visit.size.descendants )).collect::<Vec<_>>();
            assert_eq!( visits, vec![ (0, 2, 6), (1, 2, 2), (4, 2, 2), (2, 0, 0), (3, 0, 0), (5, 0, 0), (6, 0, 0), ]);
        }

        #[test] fn bfs_mut() {
            use crate::Tree;

            let mut tree = Tree::from_tuple(( 0, (1,2,3), (4,5,6), ));
            let mut root = tree.root_mut();
            root.bfs_mut().iter.zip( 1.. ).for_each( |(visit,nth)| *visit.data += 10 * nth );
            assert_eq!( tree, Tree::<i32>::from_tuple(( 10, (21,42,53), (34,65,76), )));
        }
    }

    mod forest {
        #[test] fn bfs() {
            use crate::Forest;

            let forest = Forest::from_tuple(( (1,2,3), (4,5,6), ));
            let visits = forest.bfs().iter.map( |visit| (*visit.data, visit.size.degree, visit.size.descendants )).collect::<Vec<_>>();
            assert_eq!( visits, vec![ (1, 2, 2), (4, 2, 2), (2, 0, 0), (3, 0, 0), (5, 0, 0), (6, 0, 0), ]);
        }

        #[test] fn bfs_mut() {
            use crate::Forest;
            let mut forest = Forest::<i32>::from_tuple(( (1,2,3), (4,5,6), ));
            forest.bfs_mut().iter.zip( 0.. ).for_each( |(visit,nth)| *visit.data += 10 * nth );
            assert_eq!( forest, Forest::from_tuple(( (1,(22,),(33,)), (14,(45,),(56,)), )));
        }

        #[test] fn into_bfs() {
            use crate::{Forest, Size, bfs};

            let forest = Forest::<i32>::new();
            let visits = forest.into_bfs().iter.collect::<Vec<_>>();
            assert!( visits.is_empty() );

            let forest = Forest::from_tuple(( (1,2,3), (4,5,6), ));
            let visits = forest.into_bfs().iter.collect::<Vec<_>>();
            assert_eq!( visits, vec![
                bfs::Visit{ data: 1, size: Size{ degree: 2, descendants: 2 }},
                bfs::Visit{ data: 4, size: Size{ degree: 2, descendants: 2 }},
                bfs::Visit{ data: 2, size: Size{ degree: 0, descendants: 0 }},
                bfs::Visit{ data: 3, size: Size{ degree: 0, descendants: 0 }},
                bfs::Visit{ data: 5, size: Size{ degree: 0, descendants: 0 }},
                bfs::Visit{ data: 6, size: Size{ degree: 0, descendants: 0 }},
            ]);
        }
    }

    mod tree {
        #[test] fn bfs_children_mut() {
            use crate::Tree;
            let mut tree = Tree::from_tuple(( 0, (1,2,3), (4,5,6), ));
            tree.bfs_children_mut().iter.zip( 1.. ).for_each( |(visit,nth)| *visit.data += 10 * nth );
            assert_eq!( tree, Tree::<i32>::from_tuple(( 0, (11,32,43), (24,55,66), )));
        }

        #[test] fn bfs_mut() {
            use crate::Tree;

            let mut tree = Tree::from_tuple(( 0, (1,2,3), (4,5,6), ));
            tree.bfs_mut().iter.zip( 1.. ).for_each( |(visit,nth)| *visit.data += 10 * nth );
            assert_eq!( tree, Tree::<i32>::from_tuple(( 10, (21,42,53), (34,65,76), )));
        }

        #[test] fn into_bfs() {
            use crate::{Tree, Size, bfs};

            let tree = Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6) ));
            let visits = tree.into_bfs().iter.collect::<Vec<_>>();
            assert_eq!( visits, vec![
                bfs::Visit{ data: 0, size: Size{ degree: 2, descendants: 6 }},
                bfs::Visit{ data: 1, size: Size{ degree: 2, descendants: 2 }},
                bfs::Visit{ data: 4, size: Size{ degree: 2, descendants: 2 }},
                bfs::Visit{ data: 2, size: Size{ degree: 0, descendants: 0 }},
                bfs::Visit{ data: 3, size: Size{ degree: 0, descendants: 0 }},
                bfs::Visit{ data: 5, size: Size{ degree: 0, descendants: 0 }},
                bfs::Visit{ data: 6, size: Size{ degree: 0, descendants: 0 }},
            ]);
        }
    }
}
