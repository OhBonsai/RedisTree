//! size of a tree/forest/node, including degree and descendant node count

use crate::rust::*;

/// A struct keeping the node's children count and all its descendants count
/// for resource management purpose.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Size {
    pub degree      : usize, // count of child nodes
    pub descendants : usize, // count of its descendant nodes
}

impl Add for Size {
    type Output = Self;
    fn add( self, rhs: Self ) -> Self { Size{ degree: self.degree+rhs.degree, descendants: self.descendants+rhs.descendants }}
}

impl AddAssign for Size {
    fn add_assign( &mut self, rhs: Self ) {
        *self = Size{ degree: self.degree+rhs.degree, descendants: self.descendants+rhs.descendants }
    }
}

impl Sub for Size {
    type Output = Self;
    fn sub( self, rhs: Self ) -> Self { Size{ degree: self.degree-rhs.degree, descendants: self.descendants-rhs.descendants }}
}

impl SubAssign for Size {
    fn sub_assign( &mut self, rhs: Self ) {
        *self = Size{ degree: self.degree-rhs.degree, descendants: self.descendants-rhs.descendants }
    }
}
