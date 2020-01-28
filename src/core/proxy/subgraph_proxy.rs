use crate::core::{Constrainer, Graph, ImplGraphMut, GraphMut, AddEdge, EdgeWeighted, RemoveEdge, NewVertex, RemoveVertex, ImplGraph, BaseGraph, Edge};
use crate::core::constraint::Subgraph;

///
/// A subgraph of another graph.
///
/// This proxy graph will act at if only a specific subset of the underlying graph's vertices
/// exist, filtering out all other vertices and edges incident on them.
///
pub struct SubgraphProxy<C: Constrainer>{
	/// The underlying graph
	graph: C,
	/// Which vertices are part of this subgraph
	verts: Vec<<C::Graph as Graph>::Vertex>,
	///
	/// Edges who's sources are in this subgraph but who's sinks aren't.
	///
	exit_edges: Vec<(<C::Graph as Graph>::Vertex, <C::Graph as Graph>::Vertex)>,
}

impl<C: Constrainer> SubgraphProxy<C>
{
	pub fn new(underlying: C) -> Self
	{
		Self{ graph: underlying, verts: Vec::new(), exit_edges: Vec::new()}
	}
	
	pub fn expand(&mut self, v: <C::Graph as Graph>::Vertex) -> Result<(),()>
	{
		if self.graph.graph().contains_vertex(v) {
			if !self.verts.contains(&v) {
				self.verts.push(v);
				
				// Remove any exit edge that is sinked in the vertex
				while let Some(idx) = self.exit_edges.iter().position(|e| e.sink() == v) {
					self.exit_edges.remove(idx);
				}
				
				// Add any exit edge that is sourced in the vertex
				for e in self.graph.graph().edges_incident_on(v) {
					if e.source() == v {
						if !self.verts.contains(&e.sink()) {
							self.exit_edges.push((v, e.sink()));
						}
					}
				}
			}
			Ok(())
		} else {
			Err(())
		}
	}
}

impl<C: Constrainer> Graph for SubgraphProxy<C>
{
	type Vertex = <C::Graph as Graph>::Vertex;
	type VertexWeight = <C::Graph as Graph>::VertexWeight;
	type EdgeWeight = <C::Graph as Graph>::EdgeWeight;
	type Directedness = <C::Graph as Graph>::Directedness;
	
	fn all_vertices_weighted<'a>(&'a self) -> Box<dyn 'a + Iterator<Item=
		(Self::Vertex, &'a Self::VertexWeight)>>
	{
		Box::new(self.graph.graph().all_vertices_weighted().filter(
			move|(v, _)| self.verts.contains(v)))
	}
	
	fn all_edges<'a>(&'a self) -> Box<dyn 'a + Iterator<Item=
		(Self::Vertex, Self::Vertex, &'a Self::EdgeWeight)>>
	{
		Box::new(self.graph.graph().all_edges().filter(move|(v1,v2,_)|
			self.verts.contains(v1) && self.verts.contains(v2)))
	}
}

impl<C: Constrainer + ImplGraphMut> GraphMut for SubgraphProxy<C>
	where C::Graph: GraphMut
{
	fn all_vertices_weighted_mut<'a>(&'a mut self) -> Box<dyn 'a + Iterator<Item=
		(Self::Vertex, &'a mut Self::VertexWeight)>>
	{
		let verts = &self.verts;
		let graph = self.graph.graph_mut();
		
		Box::new(graph.all_vertices_weighted_mut().filter(
			move|(v, _)| verts.contains(v)))
	}
	
	fn all_edges_mut<'a>(&'a mut self) -> Box<dyn 'a + Iterator<Item=
		(Self::Vertex, Self::Vertex, &'a mut Self::EdgeWeight)>>
	{
		let verts = &self.verts;
		let graph = self.graph.graph_mut();
		
		Box::new(graph.all_edges_mut().filter(move|(v1,v2,_)|
			verts.contains(v1) && verts.contains(v2)))
	}
}

impl<C: Constrainer + ImplGraphMut> AddEdge for SubgraphProxy<C>
	where C::Graph: AddEdge
{
	fn add_edge_weighted<E>(&mut self, e: E) -> Result<(), ()>
		where E: EdgeWeighted<Self::Vertex, Self::EdgeWeight>
	{
		if self.edge_valid((e.source(), e.sink())) {
			self.graph.graph_mut().add_edge_weighted(e)
		}else {
			Err(())
		}
	}
}

impl<C: Constrainer + ImplGraphMut> RemoveEdge for SubgraphProxy<C>
	where C::Graph: RemoveEdge
{
	fn remove_edge_where<F>(&mut self, f: F)
							-> Result<(Self::Vertex, Self::Vertex, Self::EdgeWeight), ()>
		where F: Fn((Self::Vertex, Self::Vertex, &Self::EdgeWeight)) -> bool
	{
		let verts = &self.verts;
		self.graph.graph_mut().remove_edge_where(|e|
			verts.contains(&e.0) && verts.contains(&e.1) && f(e))
	}
}

impl<C: Constrainer + ImplGraphMut> NewVertex for SubgraphProxy<C>
	where C::Graph: NewVertex
{
	fn new_vertex_weighted(&mut self, w: Self::VertexWeight) -> Result<Self::Vertex, ()> {
		let v = self.graph.graph_mut().new_vertex_weighted(w)?;
		self.verts.push(v);
		Ok(v)
	}
}

impl<C: Constrainer + ImplGraphMut> RemoveVertex for SubgraphProxy<C>
	where C::Graph: RemoveVertex
{
	fn remove_vertex(&mut self, v: Self::Vertex) -> Result<Self::VertexWeight, ()> {
		if self.contains_vertex(v) {
			let w = self.graph.graph_mut().remove_vertex(v)?;
			let index = self.verts.iter().position(|&t| t == v )
				.expect("Couldn't find removed vertex in subgraph");
			self.verts.remove(index);
			Ok(w)
		}else {
			Err(())
		}
	}
}

impl<C: Constrainer> ImplGraph for SubgraphProxy<C>
{
	type Graph = Self;
	
	fn graph(&self) -> &Self::Graph {
		self
	}
}
impl<C: Constrainer> ImplGraphMut for SubgraphProxy<C>
{
	fn graph_mut(&mut self) -> &mut Self::Graph {
		self
	}
}
impl<C: Constrainer> BaseGraph for SubgraphProxy<C> {}

impl<C: Constrainer> Subgraph for SubgraphProxy<C>
{
	fn exit_edges<'a>(&'a self) -> Box<dyn 'a + Iterator<Item=
	(Self::Vertex, Self::Vertex)>>
	{
		Box::new(self.exit_edges.iter().cloned())
	}
}