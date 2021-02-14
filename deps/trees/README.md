This project provides trees data structure serving for general purpose.

## Quickstart

Impatient readers can start with the
[notations](https://oooutlk.github.io/trees/notations.html).

# Features

1. Step-by-step
[creating, reading, updating, deleting](https://oooutlk.github.io/trees/crud.html)
and iterating nodes with assocated data items.

2. Compact notations to express trees: `-`,`/` encoded or tuple encoded trees.

3. Depth first search cursor.

4. Breadth first search iterators.

5. Trees can be built by stages, with nodes stored scatteredly among memory.

6. Trees can be built once through, with nodes stored contiguously.

7. Support exclusive ownership with static borrow check.

8. Support shared ownership with dynamic borrow check.

# Examples

- notation of a literal tree

  ```rust
  use trees::tr;

  let scattered_tree = tr(0) /( tr(1)/tr(2)/tr(3) ) /( tr(4)/tr(5)/tr(6) );
  let piled_tree = trees::Tree::from(( 0, (1,2,3), (4,5,6) ));
  ```

  They both encode a tree drawn as follows:

  ```text
  .............
  .     0     .
  .   /   \   .
  .  1     4  .
  . / \   / \ .
  .2   3 5   6.
  .............
  ```

- use tree notation to reduce syntax noise, quoted from crate `reflection_derive`, [version 0.1.1](https://github.com/oooutlk/reflection/blob/master/reflection_derive/src/lib.rs#L202):

  ```rust
  quote! {
      #(
          -( ::reflection::variant( stringify!( #vnames ))
              /(
                  #(
                      -( ::reflection::field(
                              #fnames,
                              <#ftypes1 as ::reflection::Reflection>::ty(),
                              <#ftypes2 as ::reflection::Reflection>::name(),
                              Some( <#ftypes3 as ::reflection::Reflection>::members )))
                  )*
              )
          )
      )*
  }
  ```

  The starting of tree operations are denoted by `-(` and `/(` which are humble enough to let the reader focusing on the data part.

- use iterators if the tree travesal is a "driving wheel"( you can iterate over the tree on your own ).

  ```rust
  use trees::{Node, tr};
  use std::fmt::Display;

  let tree = tr(0)
      /( tr(1) /tr(2)/tr(3) )
      /( tr(4) /tr(5)/tr(6) );

  fn tree_to_string<T:Display>( node: &Node<T> ) -> String {
      if node.has_no_child() {
          node.data.to_string()
      } else {
          format!( "{}( {})", node.data,
              node.iter().fold( String::new(),
                  |s,c| s + &tree_to_string(c) + &" " ))
      }
  }

  assert_eq!( tree_to_string( &tree ), "0( 1( 2 3 ) 4( 5 6 ) )" );
  ```

- use `TreeWalk` when the tree travesal is a "driven wheel"( driven by other library ). Quoted from crate `tsv`, [version 0.1.0](https://github.com/oooutlk/tsv/blob/master/src/de.rs#L542):

  ```rust
      fn next_value_seed<V:DeserializeSeed<'de>>( &mut self, seed: V ) -> Result<V::Value> {
          let result = self.next_element_in_row( seed )?;
          self.de.next_column();
          self.de.row += 1;
          self.de.pop_stack(); // finish key-value pair
          self.de.next_column();
          self.de.columns.revisit();
          Ok( result )
      }
  ```
  The `serde` library is driving on the schema tree when (de)serializing variables. Use `TreeWalk` methods such as `next_column` and `revisit` to follow the step.

# License

Under Apache License 2.0 or MIT License, at your will.
