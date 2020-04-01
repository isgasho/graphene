use crate::core::{
	property::{AddEdge, NewVertex, RemoveEdge, RemoveVertex},
	BaseGraph, Directedness, Edge, EdgeWeighted, Ensure, Graph, GraphDeref, GraphDerefMut,
	GraphMut,
};
use delegate::delegate;

/// A wrapper around a graph, that allows for addition and removal
/// of edges, without mutating the underlying graph.
///
/// This is useful when investigating the impact of an edge addition or removal
/// without having to actually add or remove the edge. E.g. if you only want to
/// remove an edge if some condition holds after the removal, but keep it
/// otherwise, then this proxy can be used to analyze the graph as if the edge
/// was removed.
///
/// This proxy can also be useful if the underlying graph doesn't implement edge
/// addition and removal trait. The proxy can then simulate how the graph would
/// look regardless.
///
/// If the underlying graph is mutable from the ensurer, then the edge proxy
/// can also be used to mutate vertices, however, this is done directly on the
/// underlying graph and not simulated as edge mutations are.
/// To also simulate vertex mutations, first wrap the underlying graph in
/// VertexProxy.
pub struct EdgeProxyGraph<C: Ensure>
{
	/// The underlying graph
	graph: C,
	/// Edges that have been added to the proxy and are not in the underlying
	/// graph.
	new: Vec<(<C::Graph as Graph>::Vertex, <C::Graph as Graph>::Vertex)>,
	/// Edges that have been removed from the underlying graph.
	removed: Vec<(<C::Graph as Graph>::Vertex, <C::Graph as Graph>::Vertex)>,
}

impl<C: Ensure> EdgeProxyGraph<C>
{
	pub fn new(underlying: C) -> Self
	{
		Self {
			graph: underlying,
			new: Vec::new(),
			removed: Vec::new(),
		}
	}
}

impl<C: Ensure> Graph for EdgeProxyGraph<C>
{
	type Directedness = <C::Graph as Graph>::Directedness;
	type EdgeWeight = ();
	type Vertex = <C::Graph as Graph>::Vertex;
	type VertexWeight = <C::Graph as Graph>::VertexWeight;

	delegate! {
		to self.graph.graph() {
			fn all_vertices_weighted<'a>(
				&'a self,
			) -> Box<dyn 'a + Iterator<Item = (Self::Vertex, &'a Self::VertexWeight)>>;
		}
	}

	fn all_edges<'a>(
		&'a self,
	) -> Box<dyn 'a + Iterator<Item = (Self::Vertex, Self::Vertex, &'a Self::EdgeWeight)>>
	{
		let underlying_edges = self.graph.graph().all_edges();
		let mut rem_used = Vec::with_capacity(self.removed.len());
		rem_used.extend(self.removed.iter().map(|_| false));
		let removed = underlying_edges
			.filter(move |e| {
				if let Some((idx, _)) = self.removed.iter().enumerate().find(|(idx, rem)| {
					!rem_used[*idx]
						&& ((rem.source() == e.source() && rem.sink() == e.sink())
							|| (!Self::Directedness::directed()
								&& rem.source() == e.sink() && rem.sink() == e.source()))
				})
				{
					rem_used[idx] = true;
					false
				}
				else
				{
					true
				}
			})
			.map(|e| (e.source(), e.sink(), &()));
		Box::new(
			self.new
				.iter()
				.cloned()
				.map(|e| (e.source(), e.sink(), &()))
				.chain(removed),
		)
	}
}

impl<C: Ensure + GraphDerefMut> GraphMut for EdgeProxyGraph<C>
where
	C::Graph: GraphMut,
{
	delegate! {
		to self.graph.graph_mut() {
			fn all_vertices_weighted_mut<'a>(
				&'a mut self,
			) -> Box<dyn 'a + Iterator<Item = (Self::Vertex, &'a mut Self::VertexWeight)>>;
		}
	}

	fn all_edges_mut<'a>(
		&'a mut self,
	) -> Box<dyn 'a + Iterator<Item = (Self::Vertex, Self::Vertex, &'a mut Self::EdgeWeight)>>
	{
		unimplemented!("No way to implement this as &mut () cannot be returned.")
	}
}

impl<C: Ensure> AddEdge for EdgeProxyGraph<C>
{
	fn add_edge_weighted<E>(&mut self, e: E) -> Result<(), ()>
	where
		E: EdgeWeighted<Self::Vertex, Self::EdgeWeight>,
	{
		if self.contains_vertex(e.source()) && self.contains_vertex(e.sink())
		{
			self.new.push((e.source(), e.sink()));
			Ok(())
		}
		else
		{
			Err(())
		}
	}
}

impl<C: Ensure> RemoveEdge for EdgeProxyGraph<C>
{
	fn remove_edge_where<F>(
		&mut self,
		f: F,
	) -> Result<(Self::Vertex, Self::Vertex, Self::EdgeWeight), ()>
	where
		F: Fn((Self::Vertex, Self::Vertex, &Self::EdgeWeight)) -> bool,
	{
		// First try to find a valid new vertex
		let to_remove = self
			.new
			.iter()
			.cloned()
			.enumerate()
			.find(|(_, e)| f((e.source(), e.sink(), &())));

		if let Some((idx, e)) = to_remove
		{
			self.new.remove(idx);
			Ok((e.source(), e.sink(), ()))
		}
		else
		{
			// If no new vertex is valid, look through the existing ones.
			let to_remove = self
				.all_edges()
				.map(|e| (e.source(), e.sink()))
				.find(|e| f((e.source(), e.sink(), &())));
			if let Some(e) = to_remove
			{
				self.removed.push((e.source(), e.sink()));
				Ok((e.source(), e.sink(), ()))
			}
			else
			{
				Err(())
			}
		}
	}
}

impl<C: Ensure + GraphDerefMut> NewVertex for EdgeProxyGraph<C>
where
	C::Graph: NewVertex,
{
	delegate! {
		to self.graph.graph_mut() {
			fn new_vertex_weighted(&mut self, w: Self::VertexWeight) -> Result<Self::Vertex, ()>;
		}
	}
}

impl<C: Ensure + GraphDerefMut> RemoveVertex for EdgeProxyGraph<C>
where
	C::Graph: RemoveVertex,
{
	fn remove_vertex(&mut self, v: Self::Vertex) -> Result<Self::VertexWeight, ()>
	{
		self.new.retain(|e| e.source() != v && e.sink() != v);
		self.graph.graph_mut().remove_vertex(v)
	}
}

impl<C: Ensure> GraphDeref for EdgeProxyGraph<C>
{
	type Graph = Self;

	fn graph(&self) -> &Self::Graph
	{
		self
	}
}
impl<C: Ensure> GraphDerefMut for EdgeProxyGraph<C>
{
	fn graph_mut(&mut self) -> &mut Self::Graph
	{
		self
	}
}
impl<C: Ensure> BaseGraph for EdgeProxyGraph<C> {}
