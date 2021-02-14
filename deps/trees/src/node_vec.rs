//! Buffer for storing nodes in contiguous memory allocation.

use super::{Node, Data, rc::Shared};
use crate::Size;
use crate::{TupleTree, TupleForest};

use crate::rust::*;

/// Buffer for storing nodes in contiguous memory allocation.
pub(crate) struct NodeVec<T> {
    pub(crate) buf     : Vec<Shared<RefCell<Node<T>>>>,
    pub(crate) ref_cnt : Cell<usize>, // alive node count
}

impl<T> NodeVec<T> {
    pub(crate) fn new_raw_non_null( cap: usize ) -> NonNull<NodeVec<T>> {
        unsafe {
            NonNull::new_unchecked( Box::into_raw( Box::new( NodeVec::<T> {
                buf: (0..cap).map( |_| Shared::new( RefCell::new( Node::default() ))).collect::<Vec<_>>(),
                ref_cnt: Cell::new( cap ),
            })))
        }
    }

    pub(crate) fn non_null( &self ) -> NonNull<NodeVec<T>> {
        unsafe {
            NonNull::new_unchecked( self as *const NodeVec<T> as *mut NodeVec<T> )
        }
    }

    pub(crate) fn non_null_node( &self, index: usize ) -> NonNull<Node<T>> {
        unsafe {
            NonNull::new_unchecked( self.buf.get_unchecked( index ).try_borrow_unguarded().unwrap() as *const Node<T> as *mut Node<T> )
        }
    }

    pub(crate) fn node( &self, index: usize ) -> &Node<T> { unsafe{ &*self.non_null_node( index ).as_ptr() }}
    pub(crate) fn node_mut( &mut self, index: usize ) -> &mut Node<T> { unsafe{ &mut *self.non_null_node( index ).as_ptr() }}

    pub(crate) fn make_piled_node( &mut self, parent: Option<NonNull<Node<T>>>, index: usize, data: T, size: Size ) -> NonNull<Node<T>> {
        self.make_node( parent, index, Data::Piled{ data, owner: self.non_null() }, size )
    }

    pub(crate) fn make_node( &mut self, parent: Option<NonNull<Node<T>>>, index: usize, data: Data<T>, size: Size ) -> NonNull<Node<T>> {
        unsafe {
            let node = self.buf.get_unchecked_mut( index );
            let mut node = node.deref().borrow_mut();
            node.up = parent;
            node.size = size;
            node.data = data;

            node.non_null()
        }
    }

    pub(crate) fn append_child( &mut self, parent: usize, child: usize, data: T, size: Size ) {
        let parent_node = Some( self.non_null_node( parent ));
        let mut child = self.make_piled_node( parent_node, child, data, size );

        match self.node( parent ).tail {
            Some( mut tail ) => unsafe{ tail.as_mut().connect_next( child.as_mut() )},
            None             => self.node_mut( parent ).head = Some( child ),
        }
        self.node_mut( parent ).tail = Some( child );
    }

    pub(crate) fn construct_tree<Tuple,Shape>( &mut self, tuple: Tuple )
        where Tuple: TupleTree<T,Shape>
    {
        let height = Tuple::height();
        let mut offsets = Vec::with_capacity( height );
        offsets.push( 0 );
        if height > 1 {
            offsets.push( 1 );
            for depth in 2..height {
                let offset = offsets[ (depth-1) ] + Tuple::descendants( depth-2 );
                offsets.push( offset );
            }
        }

        use crate::tuple::Visit;

        let mut is_root = true;
        let mut leaf = false;
        let mut parent = 0;
        let mut depth = 1;
        let mut f = |visit, size| {
            if is_root {
                let data = match visit {
                    Visit::Branch( data ) |
                    Visit::Leaf( data ) => data,
                    Visit::Frame => unreachable!(),
                };
                self.make_piled_node( None, 0, data, size );
                is_root = false;
            } else {
                match visit {
                    Visit::Branch( data ) => {
                        self.append_child( parent, offsets[ depth ], data, size );
                        parent = offsets[ depth ];
                        depth += 1;
                    },
                    Visit::Leaf( data ) => {
                        self.append_child( parent, offsets[ depth ], data, size );
                        offsets[ depth ] += 1;
                        if !leaf { leaf = true; }
                    },
                    Visit::Frame => {
                        depth -= 1;
                        if leaf {
                            leaf = false;
                            offsets[ depth ] += 1;
                            if depth > 0 {
                                parent = offsets[ depth-1 ];
                            }
                        }
                    },
                }
            }
        };
        Tuple::preorder_with_size_hint( tuple, &mut f );
    }

    pub(crate) fn construct_forest<Tuple,Shape>( &mut self, tuple: Tuple )
        where Tuple: TupleForest<T,Shape>
    {
        let height = Tuple::height() + 1;
        let mut offsets = Vec::with_capacity( height );
        offsets.push( 0 );
        if height > 1 {
            offsets.push( 1 );
            for depth in 2..height {
                let offset = offsets[ depth-1 ] + Tuple::descendants( depth-2 );
                offsets.push( offset );
            }
        }

        let fake_root = Data::PiledNone{ owner: unsafe{ NonNull::new_unchecked( self )}};

        self.make_node( None, 0, fake_root, Tuple::SIZE );

        let mut parent = 0;
        let mut depth = 1;
        let mut leaf = false;
        let mut f = |visit, size| {
            use crate::tuple::Visit;
            match visit {
                Visit::Branch( data ) => {
                    self.append_child( parent, offsets[ depth ], data, size );
                    parent = offsets[ depth ];
                    depth += 1;
                },
                Visit::Leaf( data ) => {
                    self.append_child( parent, offsets[ depth ], data, size );
                    offsets[ depth ] += 1;
                    if !leaf { leaf = true; }
                },
                Visit::Frame => {
                    depth -= 1;
                    if leaf {
                        leaf = false;
                        offsets[ depth ] += 1;
                        parent = offsets[ depth-1 ];
                    }
                },
            }
        };
        Tuple::preorder_with_size_hint( tuple, &mut f );
    }

    pub(crate) fn decr_ref( owner: NonNull<NodeVec<T>> ) {
        unsafe {
            let node_vec = owner.as_ref();
            use super::rc::RefCount;
            if node_vec.ref_cnt.decr() == 0 {
                drop( Box::from_raw( owner.as_ptr() ));
            }
        }
    }
}

impl<T> Drop for NodeVec<T> {
    fn drop( &mut self ) {
        unsafe{ self.buf.set_len( 0 ); }
    }
}
