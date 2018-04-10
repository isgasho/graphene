//!
//! Testing for graphene::core.
//!
//! All tests will use the GraphMock struct to test default implementations.
//! There is no need to test the required implementation, as the graph GraphMock wraps
//! is assumed to be tested in that regard.
//!
//!
//!
//!
use super::*;

use arbitraries::*;
use graphene::core::*;
use graphene::common::*;
use quickcheck::*;
use self::utilities::*;

mod base_graph;
mod utilities;
pub mod constraints;


///
/// Used to test the default implementations of Graph traits.
///
/// Wraps AdjListGraph which provides an implementation for the required methods
/// while the rest of the methods use the default implementation even though AdjListGraph
/// may have its own.
///
///
custom_graph!{
	pub struct GraphMock<V,W>
	where AdjListGraph<V,W>
}
