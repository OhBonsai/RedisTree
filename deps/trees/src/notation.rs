//! Operator overloading of `-` and `/` for constructing tree expression.

use super::{Forest, Tree};

macro_rules! impl_notations {
    ($tree:ident, $forest:ident, $tr:ident, $fr:ident) => {
        /// `tr` stands for `Tree`
        pub fn $tr<T>( data: T ) -> $tree<T> { $tree::<T>::new( data )}

        /// `fr` stands for `Forest`
        pub fn $fr<T>() -> $forest<T> { $forest::<T>::new() }

        // - $tree
        impl<T> crate::rust::Neg for $tree<T> {
            type Output = $forest<T>;

            fn neg( self ) -> $forest<T> {
                let mut forest = $fr();
                forest.push_back( self );
                forest
            }
        }

        // - &$tree
        impl<'a,T:Clone> crate::rust::Neg for &'a $tree<T> {
            type Output = $forest<T>;

            fn neg( self ) -> $forest<T> {
                let mut forest = $fr();
                forest.push_back( self.clone() );
                forest
            }
        }

        // $tree - $tree
        impl<T> crate::rust::Sub<Self> for $tree<T> {
            type Output = $forest<T>;

            fn sub( self, rhs: Self ) -> $forest<T> {
                let mut forest = $fr();
                forest.push_back( self );
                forest.push_back( rhs );
                forest
            }
        }

        // $tree - &$tree
        impl<'a,T:Clone> crate::rust::Sub<&'a $tree<T>> for $tree<T> {
            type Output = $forest<T>;

            fn sub( self, rhs: &'a $tree<T> ) -> $forest<T> {
                let mut forest = $fr();
                forest.push_back( self );
                forest.push_back( rhs.clone() );
                forest
            }
        }

        // &$tree - $tree
        impl<'a,T:Clone> crate::rust::Sub<$tree<T>> for &'a $tree<T> {
            type Output = $forest<T>;

            fn sub( self, rhs: $tree<T> ) -> $forest<T> {
                let mut forest = $fr();
                forest.push_back( self.clone() );
                forest.push_back( rhs );
                forest
            }
        }

        // &$tree - &$tree
        impl<'a,T:Clone> crate::rust::Sub<Self> for &'a $tree<T> {
            type Output = $forest<T>;

            fn sub( self, rhs: Self ) -> $forest<T> {
                let mut forest = $fr();
                forest.push_back( self.clone() );
                forest.push_back( rhs.clone() );
                forest
            }
        }

        // $tree / $forest
        impl<T> crate::rust::Div<$forest<T>> for $tree<T> {
            type Output = $tree<T>;

            fn div( mut self, rhs: $forest<T> ) -> $tree<T> {
                self.append( rhs );
                self
            }
        }

        // $tree / &$forest
        impl<'a,T:Clone> crate::rust::Div<&'a $forest<T>> for $tree<T> {
            type Output = $tree<T>;

            fn div( mut self, rhs: &'a $forest<T> ) -> $tree<T> {
                self.append( rhs.clone() );
                self
            }
        }

        // &$tree / $forest
        impl<'a,T:Clone> crate::rust::Div<$forest<T>> for &'a $tree<T> {
            type Output = $tree<T>;

            fn div( self, rhs: $forest<T> ) -> $tree<T> {
                let mut tree = self.clone();
                tree.append( rhs );
                tree
            }
        }

        // &$tree / &$forest
        impl<'a,T:Clone> crate::rust::Div<&'a $forest<T>> for &'a $tree<T> {
            type Output = $tree<T>;

            fn div( self, rhs: &'a $forest<T> ) -> $tree<T> {
                let mut tree = self.clone();
                tree.append( rhs.clone() );
                tree
            }
        }

        // $tree / $tree
        impl<T> crate::rust::Div<$tree<T>> for $tree<T> {
            type Output = $tree<T>;

            fn div( mut self, rhs: $tree<T> ) -> $tree<T> {
                self.push_back( rhs );
                self
            }
        }

        // $tree / &$tree
        impl<'a,T:Clone> crate::rust::Div<&'a $tree<T>> for $tree<T> {
            type Output = $tree<T>;

            fn div( mut self, rhs: &'a $tree<T> ) -> $tree<T> {
                self.push_back( rhs.clone() );
                self
            }
        }

        // &$tree / $tree
        impl<'a,T:Clone> crate::rust::Div<$tree<T>> for &'a $tree<T> {
            type Output = $tree<T>;

            fn div( self, rhs: $tree<T> ) -> $tree<T> {
                let mut tree = self.clone();
                tree.push_back( rhs );
                tree
            }
        }

        // &$tree / &$tree
        impl<'a,T:Clone> crate::rust::Div<Self> for &'a $tree<T> {
            type Output = $tree<T>;

            fn div( self, rhs: Self ) -> $tree<T> {
                let mut tree = self.clone();
                tree.push_back( rhs.clone() );
                tree
            }
        }

        // $tree / ()
        impl<T> crate::rust::Div<()> for $tree<T> {
            type Output = $tree<T>;

            fn div( self, _rhs: () ) -> $tree<T> {
                self
            }
        }

        // &$tree / ()
        impl<'a,T:Clone> crate::rust::Div<()> for &'a $tree<T> {
            type Output = $tree<T>;

            fn div( self, _rhs: () ) -> $tree<T> {
                self.clone()
            }
        }

        // $forest - $tree
        impl<T> crate::rust::Sub<$tree<T>> for $forest<T> {
            type Output = $forest<T>;

            fn sub( mut self, rhs: $tree<T> ) -> Self {
                self.push_back( rhs );
                self
            }
        }

        // $forest - &$tree
        impl<'a,T:Clone> crate::rust::Sub<&'a $tree<T>> for $forest<T> {
            type Output = $forest<T>;

            fn sub( mut self, rhs: &'a $tree<T> ) -> Self {
                self.push_back( rhs.clone() );
                self
            }
        }

        // &$forest - $tree
        impl<'a,T:Clone> crate::rust::Sub<$tree<T>> for &'a $forest<T> {
            type Output = $forest<T>;

            fn sub( self, rhs: $tree<T> ) -> $forest<T> {
                let mut forest = self.clone();
                forest.push_back( rhs );
                forest
            }
        }

        // &$forest - &$tree
        impl<'a,'b,T:Clone> crate::rust::Sub<&'b $tree<T>> for &'a $forest<T> {
            type Output = $forest<T>;

            fn sub( self, rhs: &'b $tree<T> ) -> $forest<T> {
                let mut forest = self.clone();
                forest.push_back( rhs.clone() );
                forest
            }
        }

        // $forest - $forest
        impl<T> crate::rust::Sub<$forest<T>> for $forest<T> {
            type Output = $forest<T>;

            fn sub( mut self, rhs: Self ) -> Self {
                self.append( rhs );
                self
            }
        }

        // $forest - &$forest
        impl<'a,T:Clone> crate::rust::Sub<&'a $forest<T>> for $forest<T> {
            type Output = $forest<T>;

            fn sub( mut self, rhs: &'a $forest<T> ) -> Self {
                self.append( rhs.clone() );
                self
            }
        }

        // &$forest - $forest
        impl<'a,T:Clone> crate::rust::Sub<$forest<T>> for &'a $forest<T> {
            type Output = $forest<T>;

            fn sub( self, mut rhs: $forest<T> ) -> $forest<T> {
                rhs.prepend( self.clone() );
                rhs
            }
        }

        // &$forest - &$forest
        impl<'a,'b,T:Clone> crate::rust::Sub<&'b $forest<T>> for &'a $forest<T> {
            type Output = $forest<T>;

            fn sub( self, rhs: &'b $forest<T> ) -> $forest<T> {
                let mut forest = self.clone();
                forest.append( rhs.clone() );
                forest
            }
        }
    };
}

impl_notations!{ Tree, Forest, tr, fr }
