use crate::core::{property::AddEdge, Ensure, Graph, GraphDerefMut};
use std::borrow::Borrow;

/// A marker trait for graphs containing no graph loops.
///
/// In graph theory, a loop is an edge that connects a vertex to itself.
/// This trait guarantees that there are no loops in the graph and that no loops
/// can be added to it.
pub trait NoLoops: Graph
{
	fn no_loops_func(&self) {}
}

pub struct NoLoopsGraph<C: Ensure>(C);

impl<C: Ensure> Ensure for NoLoopsGraph<C>
{
	fn ensure_unvalidated(c: Self::Ensured, _: ()) -> Self
	{
		Self(c)
	}

	fn validate(c: &Self::Ensured, _: &()) -> bool
	{
		c.graph().all_vertices().all(|v| {
			c.graph()
				.edges_between(v.borrow(), v.borrow())
				.next()
				.is_none()
		})
	}
}

impl<C: Ensure + GraphDerefMut> AddEdge for NoLoopsGraph<C>
where
	C::Graph: AddEdge,
{
	fn add_edge_weighted(
		&mut self,
		source: &Self::Vertex,
		sink: &Self::Vertex,
		weight: Self::EdgeWeight,
	) -> Result<(), ()>
	{
		if source == sink
		{
			Err(())
		}
		else
		{
			self.0.graph_mut().add_edge_weighted(source, sink, weight)
		}
	}
}

impl<C: Ensure> NoLoops for NoLoopsGraph<C> {}

impl_ensurer! {
	use<C> NoLoopsGraph<C>: Ensure, NoLoops, AddEdge
	as (self.0) : C
}
