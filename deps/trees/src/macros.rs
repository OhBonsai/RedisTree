macro_rules! impl_debug_display_for_collection {
    ( $ty:ident, $($agent:tt)+ ) => {
        use crate::rust::{Debug, Display};

        impl<T:Debug> Debug for $ty<T> {
            fn fmt( &self, f: &mut Formatter ) -> fmt::Result { self.$($agent)+.fmt(f) }
        }

        impl<T:Display> Display for $ty<T> {
            fn fmt( &self, f: &mut Formatter ) -> fmt::Result { write!( f, "{}", self.$($agent)+ )}
        }
    };
}

macro_rules! impl_debug_display_for_forest {
    ( $ty:ident, $($agent:tt)+ ) => {
        use crate::rust::{Debug, Display};

        impl<T:Debug> Debug for $ty<T> {
            fn fmt( &self, f: &mut Formatter ) -> fmt::Result {
                if self.has_no_child() {
                    write!( f, "()" )
                } else {
                    write!( f, "( " )?;
                    for child in self.$($agent)+ {
                        child.fmt(f)?;
                        write!( f, " " )?;
                    }
                    write!( f, ")" )
                }
            }
        }

        impl<T:Display> Display for $ty<T> {
            fn fmt( &self, f: &mut Formatter ) -> fmt::Result {
                if self.has_no_child() {
                    write!( f, "()" )
                } else {
                    write!( f, "( " )?;
                    for child in self.$($agent)+ {
                        write!( f, "{} ", child )?;
                    }
                    write!( f, ")" )
                }
            }
        }
    };
}

macro_rules! impl_order_relations_for_collection {
    ( $ty:ident, $($agent:tt)+ ) => {
        impl<T:PartialEq> PartialEq for $ty<T> {
            fn eq( &self, other: &Self ) -> bool { self.$($agent)+.eq( other.$($agent)+ )}
            fn ne( &self, other: &Self ) -> bool { self.$($agent)+.ne( other.$($agent)+ )}
        }

        impl<T:Eq> Eq for $ty<T> {}

        impl<T:PartialOrd> PartialOrd for $ty<T> {
            fn partial_cmp( &self, other: &Self ) -> Option<Ordering> { self.$($agent)+.partial_cmp( other.$($agent)+ )}
        }

        impl<T:Ord> Ord for $ty<T> {
            fn cmp( &self, other: &Self ) -> Ordering { self.$($agent)+.cmp( other.$($agent)+ )}
        }
    };
}

macro_rules! impl_hash_for_collection {
    ( $ty:ident, $($agent:tt)+ ) => {
        use crate::rust::{Hash, Hasher};

        impl<T:Hash> Hash for $ty<T> {
            fn hash<H:Hasher>( &self, state: &mut H ) { self.$($agent)+.hash( state )}
        }
    };
}

macro_rules! impl_hash_for_forest {
    ( $ty:ident, $($agent:tt)+ ) => {
        use crate::rust::{Hash, Hasher};

        impl<T:Hash> Hash for $ty<T> {
            fn hash<H:Hasher>( &self, state: &mut H ) {
                for child in self.$($agent)+ {
                    child.hash( state )
                }
            }
        }
    };
}

macro_rules! impl_debug_display_for_node {
    ( $ty:ident, $iter:ident, $($data:tt)+ ) => {
        use crate::rust::{Debug, Display};

        impl<T:Debug> Debug for $ty<T> {
            fn fmt( &self, f: &mut Formatter ) -> fmt::Result {
                if self.has_no_child() {
                    self.$($data)+.fmt(f)?;
                    write!( f, "@{:?} ", self as *const _ )
                } else {
                    self.$($data)+.fmt(f)?;
                    write!( f, "@{:?} ", self as *const _ )?;
                    write!( f, "( " )?;
                    for child in self.$iter() {
                        child.fmt(f)?;
                    }
                    write!( f, ")" )
                }
            }
        }

        impl<T:Display> Display for $ty<T> {
            fn fmt( &self, f: &mut Formatter ) -> fmt::Result {
                if self.has_no_child() {
                    write!( f, "{}", self.$($data)+ )
                } else {
                    write!( f, "{}", self.$($data)+ )?;
                    write!( f, "( " )?;
                    for child in self.$iter() {
                        write!( f, "{} ", child )?;
                    }
                    write!( f, ")" )
                }
            }
        }
    };
}

macro_rules! impl_order_relations_for_node {
    ( $ty:ident, $iter:ident, $($data:tt)+ ) => {
        impl<T:PartialEq> PartialEq for $ty<T> {
            fn eq( &self, other: &Self ) -> bool { self.$($data)+ == other.$($data)+ && self.$iter().eq( other.$iter() )}
            fn ne( &self, other: &Self ) -> bool { self.$($data)+ != other.$($data)+ || self.$iter().ne( other.$iter() )}
        }

        impl<T:Eq> Eq for $ty<T> {}

        impl<T:PartialOrd> PartialOrd for $ty<T> {
            fn partial_cmp( &self, other: &Self ) -> Option<Ordering> {
                match self.$($data)+.partial_cmp( &other.$($data)+ ) {
                    None          => None,
                    Some( order ) => match order {
                        Less    => Some( Less ),
                        Greater => Some( Greater ),
                        Equal   => self.$iter().partial_cmp( other.$iter() ),
                    },
                }
            }
        }

        impl<T:Ord> Ord for $ty<T> {
            fn cmp( &self, other: &Self ) -> Ordering {
                match self.$($data)+.cmp( &other.$($data)+ ) {
                    Less    => Less,
                    Greater => Greater,
                    Equal   => self.$iter().cmp( other.$iter() ),
                }
            }
        }
    };
}

macro_rules! impl_hash_for_node {
    ( $ty:ident, $iter:ident, $($data:tt)+ ) => {
        impl<T:Hash> Hash for $ty<T> {
            fn hash<H:Hasher>( &self, state: &mut H ) {
                self.$($data)+.hash( state );
                for child in self.$iter() {
                    child.hash( state );
                }
            }
        }
    };
}
