// Copyright 2018 oooutlk@outlook.com. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! General purpose tree library.
//! See the [trees book](https://oooutlk.github.io/trees/) for more.
//!
//! # Examples
//!
//! The code below construct the following tree in different ways:
//!
//! ```text
//! .............
//! .     0     .
//! .   /   \   .
//! .  1     4  .
//! . / \   / \ .
//! .2   3 5   6.
//! .............
//! ```
//!
//! ## Example of `tr` notations for building trees
//!
//! ```rust
//! use trees::tr;
//!
//! let tree = tr(0) /( tr(1)/tr(2)/tr(3) ) /( tr(4)/tr(5)/tr(6) );
//! ```
//!
//! ## Example of tuple notations for building trees
//!
//! ```rust
//! let tree = trees::Tree::<i32>::from_tuple(( 0, (1,2,3), (4,5,6) ));
//! ```
//!
//! ## Example of building trees step by step
//!
//! ```rust
//! use trees::Tree;
//!
//! let mut tree = Tree::new(0);
//!
//! let mut root = tree.root_mut();
//! root.push_back( Tree::new(1) );
//! root.push_back( Tree::new(4) );
//!
//! let mut children = root.iter_mut();
//!
//! let mut node_1 = children.next().unwrap();
//! node_1.push_back( Tree::new(2) );
//! node_1.push_back( Tree::new(3) );
//!
//! let mut node_4 = children.next().unwrap();
//! node_4.push_back( Tree::new(5) );
//! node_4.push_back( Tree::new(6) );
//! ```
//!
//! # Overview of features
//!
//! 1. Step-by-step [creating, reading, updating, deleting](./crud.md) and iterating
//! nodes with assocated data items.
//!
//! 2. Compact notations to express trees: `-`,`/` encoded or tuple encoded trees.
//!
//! 3. Depth first search cursor.
//!
//! 4. Breadth first search iterators.
//!
//! 5. Trees can be built by stages, with nodes stored scatteredly among memory.
//!
//! 6. Trees can be built once through, with nodes stored contiguously.
//!
//! 7. Support exclusive ownership with static borrow check.
//!
//! 8. Support shared ownership with dynamic borrow check.

#![cfg_attr( feature = "no_std", no_std )]

#[doc( hidden )]
pub mod rust {
    #[cfg(not(feature="no_std"))] pub use std::borrow::{Borrow, ToOwned};
    #[cfg(not(feature="no_std"))] pub use std::boxed::Box;
    #[cfg(not(feature="no_std"))] pub use std::cell::{Cell, Ref, RefMut, RefCell};
    #[cfg(not(feature="no_std"))] pub use std::collections::VecDeque;
    #[cfg(not(feature="no_std"))] pub use std::cmp::Ordering::{self, *};
    #[cfg(not(feature="no_std"))] pub use std::fmt::{self, Debug, Display, Formatter};
    #[cfg(not(feature="no_std"))] pub use std::hash::{Hasher, Hash};
    #[cfg(not(feature="no_std"))] pub use std::iter::{Iterator, FromIterator, IntoIterator, FusedIterator};
    #[cfg(not(feature="no_std"))] pub use std::marker::{PhantomData, Unpin};
    #[cfg(not(feature="no_std"))] pub use std::mem::{self, forget, transmute, MaybeUninit};
    #[cfg(not(feature="no_std"))] pub use std::ops::{Add, AddAssign, Deref, DerefMut, Div, Neg, Sub, SubAssign};
    #[cfg(not(feature="no_std"))] pub use std::pin::Pin;
    #[cfg(not(feature="no_std"))] pub use std::ptr::{self, NonNull, null, null_mut};
    #[cfg(not(feature="no_std"))] pub use std::rc::{Rc, Weak};
    #[cfg(not(feature="no_std"))] pub use std::vec::Vec;

    #[cfg(feature="no_std")] extern crate core;
    #[cfg(feature="no_std")] extern crate alloc;
    #[cfg(feature="no_std")] pub use self::alloc::borrow::{Borrow, ToOwned};
    #[cfg(feature="no_std")] pub use self::alloc::boxed::Box;
    #[cfg(feature="no_std")] pub use self::alloc::string::String;
    #[cfg(feature="no_std")]
                #[cfg(test)] pub use self::alloc::string::ToString;
    #[cfg(feature="no_std")] pub use self::alloc::collections::VecDeque;
    #[cfg(feature="no_std")]
                #[cfg(test)] pub use self::alloc::format;
    #[cfg(feature="no_std")] pub use self::alloc::rc::{Rc, Weak};
    #[cfg(feature="no_std")]
                #[cfg(test)] pub use self::alloc::vec;
    #[cfg(feature="no_std")] pub use self::alloc::vec::Vec;
    #[cfg(feature="no_std")] pub use core::cell::{Cell, Ref, RefMut, RefCell};
    #[cfg(feature="no_std")] pub use core::cmp::Ordering::{self, *};
    #[cfg(feature="no_std")] pub use core::fmt::{self, Debug, Display, Formatter};
    #[cfg(feature="no_std")] pub use core::hash::{Hasher, Hash};
    #[cfg(feature="no_std")] pub use core::iter::{Iterator, FromIterator, IntoIterator, FusedIterator};
    #[cfg(feature="no_std")] pub use core::marker::{PhantomData, Unpin};
    #[cfg(feature="no_std")] pub use core::mem::{self, forget, transmute, MaybeUninit};
    #[cfg(feature="no_std")] pub use core::ops::{Add, AddAssign, Deref, DerefMut, Div, Neg, Sub, SubAssign};
    #[cfg(feature="no_std")] pub use core::pin::Pin;
    #[cfg(feature="no_std")] pub use core::ptr::{self, NonNull, null, null_mut};
}

#[macro_use]
mod macros;

pub mod tuple;
pub use tuple::{TupleForest, TupleTree};

pub mod bfs;

pub mod size;
pub use size::Size;

pub mod tree;
pub use tree::Tree;

pub mod forest;
pub use forest::Forest;

pub mod node;
pub use node::Node;
pub(crate) use node::Data;

pub(crate) mod node_vec;
pub(crate) use node_vec::NodeVec;

pub mod iter;
pub use iter::{Iter, IterMut};
pub(crate) use iter::CountedRawIter;

pub mod into_iter;
pub use into_iter::IntoIter;

pub mod heap;

pub mod walk;
pub use walk::{TreeWalk, ForestWalk};

pub mod notation;
pub use notation::{tr, fr};

pub mod iter_rc;
pub use iter_rc::IterRc;

pub mod rc;
pub use rc::{RcNode, WeakNode};

pub(crate) mod bfs_impls;


#[derive(Debug)]
pub struct Error {
    pub msg: String,
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error { msg: e }
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Error { msg: e.to_string() }
    }
}

use std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

use std::error;
impl error::Error for Error {
    fn description(&self) -> &str {
        &self.msg
    }
}



use std::pin::Pin;
impl<T> Node<T> {
    pub fn locate_first_by_path<'s, 't>(&'s self, mut path: impl Iterator<Item=&'t T> + Clone ) -> Option<&'s Node<T>>
        where T: 't + PartialEq
    {
        if let Some( data ) = path.next() {
            if self.data() == data {
                let clone_path = path.clone();

                if path.next().is_none() {
                    return Some(self)
                }

                for child in self.iter() {
                    if let Some( node ) = child.locate_first_by_path(clone_path.clone() ) {
                        return Some( node );
                    }
                }
            }
        }
        None
    }

    pub fn locate_first_by_data<'s, 't>(&'s self, data: &'t T) -> Option<&'s Node<T>>
        where T: 't + PartialEq
    {
        if self.data() == data {
           return Some(self)
        }

        for child in self.iter() {
            if let Some(node) = child.locate_first_by_data(data) {
                return Some(node);
            }
        }

        None
    }

    pub fn locate_first_mut_by_data<'s, 't>(&'s mut self, data: &'t T) ->  Option<Pin<&'s mut Node<T>>>
        where T: 't + PartialEq
    {
        if self.data() == data {
            return Some( unsafe { Pin::new_unchecked(self)});
        }

        for child in self.iter_mut() {
            let child = unsafe{ Pin::get_unchecked_mut(child) };
            if let Some(node) = child.locate_first_mut_by_data(data) {
                return Some(node);
            }
        }
        None
    }

    pub fn locate_first_mut_by_path<'s, 't>(&'s mut self, mut path: impl Iterator<Item=&'t T> + Clone ) -> Option<Pin<&'s mut Node<T>>>
        where T: 't + PartialEq
    {
        if let Some( data ) = path.next() {

            if self.data() == data {
                let clone_path = path.clone();

                if path.next().is_none() {
                    return Some( unsafe{ Pin::new_unchecked( self )});
                }

                for child in self.iter_mut() {
                    let child = unsafe{ Pin::get_unchecked_mut( child )};
                    if let Some( node ) = child.locate_first_mut_by_path( clone_path.clone() ) {
                        return Some( node );
                    }
                }
            }
        }
        None
    }


    pub fn ancestors(&self) -> Vec<&T> {
        let mut ancestors = vec![];

        let mut current_node = self;
        while let Some(node) = current_node.parent(){
            ancestors.push(node.data());
            current_node = node;
        }
        ancestors
    }


    pub fn descendants(&self) -> Vec<&T> {
        self.bfs().iter.map(|v| {
            v.data
        }).collect::<Vec<_>>()
    }


    pub fn children(&self) -> Vec<&T> {
        self.iter().map(|v| v.data()).collect::<Vec<_>>()
    }

    pub fn father(&self) -> Option<&T> {
        self.parent().map(|v| v.data())
    }

}



use std::convert::{TryFrom};
use crate::rust::Formatter;


impl TryFrom<&str> for Tree<String> {
    type Error = Error;
    fn try_from(item: &str) -> Result<Self, Self::Error> {
        Tree::<String>::try_from(item.to_string())
    }
}

impl TryFrom<String> for Tree<String> {
    type Error = Error;

    fn try_from(item: String) -> Result<Self, Self::Error> {
        let tree_string = item.trim();

        if tree_string.starts_with("(") {
            return Err("no root in tree string".into())
        }


        let mut tokens = Vec::new();
        let mut legal = 0;

        let mut t = String::from("");
        tree_string.chars().for_each(|v| {
            match v {
                '(' | ')' | ' ' => {
                    if !t.is_empty() {
                        tokens.push(t.clone());
                        t = "".to_string();
                    }
                    if v !=' ' {
                        tokens.push(v.to_string());
                    }
                    legal = if v == '(' {
                        legal + 1
                    } else if v == ')' {
                        legal - 1
                    } else {
                        legal
                    }
                },
                _ => t.push(v)
            }
        });

        if !t.is_empty() { tokens.push(t) }

        // the number of '(' is not equal to the number of ')'
        if legal !=0 || tokens.len() == 0 {
            return Err("() is not closed".into())
        }

        let mut tree = Tree::new(String::from(&tokens[0]));
        let mut forests: Vec<Forest<String>> = Vec::new();
        tokens.iter().skip(1).for_each(|v| {
            match v.as_str() {
                "(" => forests.push(Forest::new()),
                ")" => {
                    let last_forest = forests.pop().unwrap();
                    if let Some(father_forest) = forests.last_mut() {
                        last_forest.into_iter().for_each(|v| {
                            father_forest.back_mut().unwrap().push_back(v)
                        })
                    } else {
                        // stack emtpy, append forest to root
                        tree.root_mut().prepend(last_forest)
                    }
                },
                _ => forests.last_mut().unwrap().push_back(Tree::new(String::from(v)))
            }
        });
        Ok(tree)
    }
}


#[cfg(test)]
mod extend_tests {
    use super::*;


    #[test] fn test_try_from_string() {
        let tree_string = "   0( 1( 2 3bc) 4( 5 6 ) )  ";
        let wrong_string = " ((0)";

        assert!(Tree::try_from(wrong_string).is_err());
        assert!(Tree::try_from("a").is_ok());
        assert!(Tree::try_from(String::from(tree_string)).is_ok());

        assert_eq!(Tree::try_from("a").unwrap(), Tree::new("a".to_string()));


        let wrong_string = " (0)";
        assert!(Tree::try_from(wrong_string).is_err());
    }


    #[test] fn test_node_locate_by_path() {
        let mut tree = tr(0) /(tr(1)/tr(2)) /(tr(3)/tr(4));
        let path = vec![ 0,3 ];
        assert_eq!(tree.root().locate_first_by_path( path.iter() ).unwrap().data(), &3 );


        let mut root = tree.root_mut();
        let path = vec![ 0,3 ];
        let mut node3 = root.locate_first_mut_by_path(path.iter()).unwrap();
        node3.push_back(tr(4));

        println!("{:?}", tree.to_string())
    }

    #[test] fn test_node_locate_by_data() {
        let mut t = Tree::try_from("   0( 1( 2 3bc) 4( 5 6 ) )  ".to_owned()).unwrap();
        assert!(t.root().locate_first_by_data(&"3bc".to_string()).is_some());


        let node = t.root().locate_first_by_data(&"2bc".to_string());
        assert!(node.is_none());


        let mut root = t.root_mut();
        let mut node = root.locate_first_mut_by_data(&"3bc".to_string());
        assert!(node.is_some());

    }

    #[test] fn test_ancestors() {
        let mut t = Tree::try_from("   0( 1( 2 3bc) 4( 5 6 ) )  ".to_owned()).unwrap();
        println!("{:?}", t.root().locate_first_by_data(&"3bc".to_string()).unwrap().ancestors());
        println!("{:?}", t.to_string());
    }

    #[test] fn test_descendants() {
        let mut t = Tree::try_from("   0( 1( 2 3bc) 4( 5 6 ) )  ".to_owned()).unwrap();
        println!("{:?}", t.root().locate_first_by_data(&"1".to_string()).unwrap().descendants());
        println!("{:?}", t.to_string());
    }


}