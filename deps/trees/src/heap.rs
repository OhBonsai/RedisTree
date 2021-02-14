#![doc( hidden )]

use super::{Data, Node, NodeVec, Size};

use crate::rust::*;

pub(crate) fn make_node<T>( data: Data<T> ) -> NonNull<Node<T>> {
    let rc = Rc::new( RefCell::new( Node{
        prev : None,
        next : None,
        head : None,
        tail : None,
        up   : None,
        size : Size::default(),
        data ,
    }));
    let rc_raw = Rc::into_raw( rc );
    let rc_raw_non_null = unsafe{ NonNull::new_unchecked( rc_raw as *mut _ )};
    let mut rc = unsafe{ Rc::from_raw( rc_raw )};
    let node = Rc::get_mut( &mut rc ).unwrap().get_mut();
    match &mut node.data {
        Data::ScatteredNone{ owner     } |
        Data::Scattered    { owner, .. } => *owner = rc_raw_non_null,
        _ => (),
    }
    let node_raw_non_null = unsafe{ NonNull::new_unchecked( node )};

    mem::forget( rc );
    node_raw_non_null
}

pub(crate) fn drop_node<T>( mut node: NonNull<Node<T>> ) {
    unsafe {
        match node.as_mut().data.replace( Data::None ) {
            Data::None => (),
            Data::ScatteredNone{ owner     } |
            Data::Scattered    { owner, .. } => {
                drop( Rc::from_raw( owner.as_ptr() ));
            },
            Data::PiledNone    { owner     } |
            Data::Piled        { owner, .. } => {
                NodeVec::decr_ref( owner );
            },
        }
    }
}
