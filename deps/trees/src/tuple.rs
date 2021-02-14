//! Traits for implementing tuple notations

use crate::Size;

use crate::rust::*;

/// Visit one node in tree/forest building process, using tuple notations.
#[derive( Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash )]
pub enum Visit<T> {
    Branch( T ),
    Leaf(   T ),
    Frame,
}

/// For tuple notations to construct `Tree`.
pub trait TupleTree<T,Shape>: Sized {
    const SIZE: Size;
    fn descendants( indirect_level: usize ) -> usize;
    fn height() -> usize;
    fn preorder(  self, f: &mut impl FnMut( Visit<T> ));
    fn preorder_with_size_hint( self, f: &mut impl FnMut( Visit<T>, Size ));
    fn postorder( self, f: &mut impl FnMut( Visit<T> ));
    fn postorder_with_size_hint(  self, f: &mut impl FnMut( Visit<T>, Size ));
}

impl<T> TupleTree<T,()> for T {
    const SIZE: Size = Size{ degree: 0, descendants: 0 };
    fn descendants( _indirect_level: usize ) -> usize { 0 }
    fn height() -> usize { 1 }

    fn preorder( self, f: &mut impl FnMut( Visit<T> ) ) {
        f( Visit::Leaf( self ));
    }

    fn preorder_with_size_hint( self, f: &mut impl FnMut( Visit<T>, Size )) {
        f( Visit::Leaf( self ), Size::default() );
    }

    fn postorder( self, f: &mut impl FnMut( Visit<T> ) ) {
        f( Visit::Leaf( self ));
    }

    fn postorder_with_size_hint( self, f: &mut impl FnMut( Visit<T>, Size )) {
        f( Visit::Leaf( self ), Size::default() );
    }
}

macro_rules! impl_tuple_tree {
    ($len:expr => ($($n:tt $name:ident $shape:ident)*)) => {
        impl<T$(,$name,$shape)*> TupleTree<T,((),$($shape),*)> for (T,$($name,)*)
            where T: TupleTree<T,()> $(,$name: TupleTree<T,$shape>)*
        {
            const SIZE: Size = Size {
                degree     : $len,
                descendants: 0 $(+ <$name as TupleTree<T,$shape>>::SIZE.descendants+1 )*,
            };

            fn descendants( indirect_level: usize ) -> usize {
                if indirect_level == 0 {
                    $len
                } else {
                    0 $( + <$name as TupleTree<T,$shape>>::descendants( indirect_level-1 ) )*
                }
            }

            fn height() -> usize {
                1 + *[ 0 $(, <$name as TupleTree<T,$shape>>::height() )* ].iter().max_by( |x,y| x.cmp(y) ).unwrap()
            }

            fn preorder( self, f: &mut impl FnMut( Visit<T> ) ) {
                if <Self as TupleTree<T,((),$($shape),*)>>::SIZE.degree == 0 {
                    f( Visit::Leaf( self.0 ));
                } else {
                    f( Visit::Branch( self.0 ));
                    $( (self.$n).preorder( f ); )*
                    f( Visit::Frame );
                }
            }

            fn preorder_with_size_hint( self, f: &mut impl FnMut( Visit<T>, Size )) {
                let size = <Self as TupleTree<T,((),$($shape),*)>>::SIZE;
                if size.degree == 0 {
                    f( Visit::Leaf( self.0 ), size );
                } else {
                    f( Visit::Branch( self.0 ), size );
                    $( (self.$n).preorder_with_size_hint( f ); )*
                    f( Visit::Frame, size );
                }
            }

            fn postorder( self, f: &mut impl FnMut( Visit<T> ) ) {
                if <Self as TupleTree<T,((),$($shape),*)>>::SIZE.degree == 0 {
                    f( Visit::Leaf( self.0 ));
                } else {
                    f( Visit::Frame );
                    $( (self.$n).postorder( f ); )*
                    f( Visit::Branch( self.0 ));
                }
            }

            fn postorder_with_size_hint( self, f: &mut impl FnMut( Visit<T>, Size )) {
                let size = <Self as TupleTree<T,((),$($shape),*)>>::SIZE;
                if size.degree == 0 {
                    f( Visit::Leaf( self.0 ), size );
                } else {
                    f( Visit::Branch( self.0 ), size );
                    $( (self.$n).postorder_with_size_hint( f ); )*
                    f( Visit::Frame, size );
                }
            }
        }
    }
}

/// For tuple notations to construct `Forest`.
pub trait TupleForest<T,Shape>: Sized {
    const SIZE: Size;
    fn descendants( indirect_level: usize ) -> usize;
    fn height() -> usize;
    fn preorder(  self, f: &mut impl FnMut( Visit<T> ));
    fn preorder_with_size_hint(  self, f: &mut impl FnMut( Visit<T>, Size ));
    fn postorder( self, f: &mut impl FnMut( Visit<T> ));
    fn postorder_with_size_hint(  self, f: &mut impl FnMut( Visit<T>, Size ));
}

macro_rules! impl_tuple_forest {
    ($len:expr => ($($n:tt $name:ident $shape:ident)*)) => {
        impl<T,$($name,$shape),*> TupleForest<T,((),$($shape,)*)> for ($($name,)*)
            where T: TupleTree<T,()> $(,$name: TupleTree<T,$shape>)*
        {
            const SIZE: Size = Size {
                degree     : $len,
                descendants: 0 $(+ <$name as TupleTree<T,$shape>>::SIZE.descendants+1 )*,
            };

            fn descendants( indirect_level: usize ) -> usize {
                if indirect_level == 0 {
                    $len
                } else {
                    0 $( + <$name as TupleTree<T,$shape>>::descendants( indirect_level-1 ) )*
                }
            }

            fn height() -> usize {
                0 + *[ 0 $(, <$name as TupleTree<T,$shape>>::height() )* ].iter().max_by( |x,y| x.cmp(y) ).unwrap()
            }

            fn preorder( self, _f: &mut impl FnMut( Visit<T> ) ) {
                $( (self.$n).preorder( _f ); )*
            }

            fn preorder_with_size_hint(  self, _f: &mut impl FnMut( Visit<T>, Size )) {
                $( (self.$n).preorder_with_size_hint( _f ); )*
            }

            fn postorder( self, _f: &mut impl FnMut( Visit<T> ) ) {
                $( (self.$n).postorder( _f ); )*
            }

            fn postorder_with_size_hint(  self, _f: &mut impl FnMut( Visit<T>, Size )) {
                $( (self.$n).postorder_with_size_hint( _f ); )*
            }
        }
    }
}

macro_rules! tuple_tree_impls {
    ($($len:expr => ($($n:tt $name:ident $shape:ident)*))+) => {$(
        impl_tuple_tree!( $len => ($($n $name $shape)*) );
    )+};
}

tuple_tree_impls! {
    0 => ()
    1 => (1 T1 S1)
    2 => (1 T1 S1 2 T2 S2)
    3 => (1 T1 S1 2 T2 S2 3 T3 S3)
    4 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4)
    5 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5)
    6 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6)
    7 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7)
    8 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8)
    9 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9)
   10 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10)
   11 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11)
   12 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12)
   13 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13)
   14 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14)
   15 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15)
   16 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16)
   17 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17)
   18 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18)
   19 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19)
   20 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20)
   21 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21)
   22 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22)
   23 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23)
   24 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24)
   25 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25)
   26 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25 26 T26 S26)
   27 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25 26 T26 S26 27 T27 S27)
   28 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25 26 T26 S26 27 T27 S27 28 T28 S28)
   29 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25 26 T26 S26 27 T27 S27 28 T28 S28 29 T29 S29)
   30 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25 26 T26 S26 27 T27 S27 28 T28 S28 29 T29 S29 30 T30 S30)
   31 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25 26 T26 S26 27 T27 S27 28 T28 S28 29 T29 S29 30 T30 S30 31 T31 S31)
   32 => (1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25 26 T26 S26 27 T27 S27 28 T28 S28 29 T29 S29 30 T30 S30 31 T31 S31 32 T32 S32)
}

macro_rules! tuple_forest_impls {
    ($($len:expr => ($($n:tt $name:ident $shape:ident)*))+) => {$(
        impl_tuple_forest!( $len => ($($n $name $shape)*) );
    )+};
}

tuple_forest_impls! {
    0 => ()
    1 => (0 T0 S0)
    2 => (0 T0 S0 1 T1 S1)
    3 => (0 T0 S0 1 T1 S1 2 T2 S2)
    4 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3)
    5 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4)
    6 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5)
    7 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6)
    8 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7)
    9 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8)
    10 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9)
   11 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10)
   12 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11)
   13 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12)
   14 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13)
   15 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14)
   16 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15)
   17 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16)
   18 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17)
   19 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18)
   20 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19)
   21 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20)
   22 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21)
   23 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22)
   24 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23)
   25 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24)
   26 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25)
   27 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25 26 T26 S26)
   28 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25 26 T26 S26 27 T27 S27)
   29 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25 26 T26 S26 27 T27 S27 28 T28 S28)
   30 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25 26 T26 S26 27 T27 S27 28 T28 S28 29 T29 S29)
   31 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25 26 T26 S26 27 T27 S27 28 T28 S28 29 T29 S29 30 T30 S30)
   32 => (0 T0 S0 1 T1 S1 2 T2 S2 3 T3 S3 4 T4 S4 5 T5 S5 6 T6 S6 7 T7 S7 8 T8 S8 9 T9 S9 10 T10 S10 11 T11 S11 12 T12 S12 13 T13 S13 14 T14 S14 15 T15 S15 16 T16 S16 17 T17 S17 18 T18 S18 19 T19 S19 20 T20 S20 21 T21 S21 22 T22 S22 23 T23 S23 24 T24 S24 25 T25 S25 26 T26 S26 27 T27 S27 28 T28 S28 29 T29 S29 30 T30 S30 31 T31 S31)
}

#[cfg( test )]
mod tests {
    use super::*;

    #[test]
    fn tree_preorder() {
        let mut visits = Vec::new();
        let tree = (0, (1,3,4), (2,5,6), );
        TupleTree::<i32,_>::preorder( tree, &mut |visit| visits.push( visit ));
        assert_eq!( visits, vec![
            Visit::Branch(0),
            Visit::Branch(1),
            Visit::Leaf(3),
            Visit::Leaf(4),
            Visit::Frame,
            Visit::Branch(2),
            Visit::Leaf(5),
            Visit::Leaf(6),
            Visit::Frame,
            Visit::Frame,
        ]);
    }

    #[test]
    fn forest_preorder() {
        let mut visits = Vec::new();
        let forest = ( (1,3,4), (2,5,6), );
        TupleForest::<i32,_>::preorder( forest, &mut |visit| visits.push( visit ));
        assert_eq!( visits, vec![
            Visit::Branch(1),
            Visit::Leaf(3),
            Visit::Leaf(4),
            Visit::Frame,
            Visit::Branch(2),
            Visit::Leaf(5),
            Visit::Leaf(6),
            Visit::Frame,
        ]);
    }
    #[test]
    fn tree_postorder() {
        let mut visits = Vec::new();
        let tree = (0, (1,3,4), (2,5,6), );
        TupleTree::<i32,_>::postorder( tree, &mut |visit| visits.push( visit ));
        assert_eq!( visits, vec![
            Visit::Frame,
            Visit::Frame,
            Visit::Leaf(3),
            Visit::Leaf(4),
            Visit::Branch(1),
            Visit::Frame,
            Visit::Leaf(5),
            Visit::Leaf(6),
            Visit::Branch(2),
            Visit::Branch(0),
        ]);
    }

    #[test]
    fn forest_postorder() {
        let mut visits = Vec::new();
        let forest = ( (1,3,4), (2,5,6), );
        TupleForest::<i32,_>::postorder( forest, &mut |visit| visits.push( visit ));
        assert_eq!( visits, vec![
            Visit::Frame,
            Visit::Leaf(3),
            Visit::Leaf(4),
            Visit::Branch(1),
            Visit::Frame,
            Visit::Leaf(5),
            Visit::Leaf(6),
            Visit::Branch(2),
        ]);
    }
}
