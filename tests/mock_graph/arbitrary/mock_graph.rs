use crate::mock_graph::{
	arbitrary::{GuidedArbGraph, Limit},
	MockEdgeWeight, MockGraph, MockT, MockVertex, MockVertexWeight,
};
use graphene::core::{
	constraint::{AddEdge, DirectedGraph, NewVertex, RemoveEdge, RemoveVertex},
	Constrainer, Directedness, Edge, EdgeDeref, EdgeWeighted, Graph,
};
use quickcheck::{Arbitrary, Gen};
use rand::Rng;
use std::{collections::HashSet, ops::RangeBounds};

impl Arbitrary for MockVertex
{
	fn arbitrary<G: Gen>(g: &mut G) -> Self
	{
		Self {
			value: usize::arbitrary(g),
		}
	}

	fn shrink(&self) -> Box<dyn Iterator<Item = Self>>
	{
		Box::new(self.value.shrink().map(|v| Self { value: v }))
	}
}

impl Arbitrary for MockT
{
	fn arbitrary<G: Gen>(g: &mut G) -> Self
	{
		Self {
			value: u32::arbitrary(g),
		}
	}

	fn shrink(&self) -> Box<dyn Iterator<Item = Self>>
	{
		Box::new(self.value.shrink().map(|v| Self { value: v }))
	}
}

impl<D: Directedness> GuidedArbGraph for MockGraph<D>
{
	fn arbitrary_guided<G: Gen>(
		g: &mut G,
		v_range: impl RangeBounds<usize>,
		e_range: impl RangeBounds<usize>,
	) -> Self
	{
		let (v_min, v_max, e_min, e_max) = Self::validate_ranges(g, v_range, e_range);
		let mut graph = Self::empty();

		// Decide the amount of vertices
		let vertex_count = g.gen_range(v_min, v_max);

		// If the amount of vertices is 0, no edges can be created.
		if vertex_count > 0
		{
			// Create vertices
			for _ in 0..vertex_count
			{
				graph
					.new_vertex_weighted(MockVertexWeight::arbitrary(g))
					.unwrap();
			}
			let vertices = graph.all_vertices().collect::<Vec<_>>();

			// Decide the amount of edges
			let edge_count = g.gen_range(e_min, e_max);

			// Create edges
			// For each edge
			for _ in 0..edge_count
			{
				// Create a valid edge
				let t_source = vertices[g.gen_range(0, vertex_count)];
				let t_sink = vertices[g.gen_range(0, vertex_count)];
				let t_weight = MockEdgeWeight::arbitrary(g);
				graph
					.add_edge_weighted((t_source, t_sink, t_weight))
					.unwrap();
			}
		}
		graph
	}

	fn shrink_guided(&self, limits: HashSet<Limit>) -> Box<dyn Iterator<Item = Self>>
	{
		// Base case
		if self.vertices.len() == 0
		{
			return Box::new(std::iter::empty());
		}

		let mut result = Vec::new();

		// Shrink by shrinking vertex weight
		self.vertices
            .iter()
            //Get all possible shrinkages
            .flat_map(|(v, weight)| weight.shrink().map(move |shrunk| (v, shrunk)))
            //For each shrunk weight,
            //create a new graph where the vertex has that weight
            .for_each(|(v, shrunk_weight)| {
                let mut new_graph = self.clone();
                new_graph.vertices.insert(*v, shrunk_weight);
                result.push(new_graph);
            });

		// Shrink by shrinking edge weight
		self.all_edges().for_each(|(source, sink, ref weight)| {
			let shrunk_weights = weight.shrink();

			shrunk_weights.for_each(|s_w| {
				let mut shrunk_graph = self.clone();
				shrunk_graph
					.remove_edge_where_weight((source, sink), |ref w| w.value == weight.value)
					.unwrap();
				shrunk_graph.add_edge_weighted((source, sink, s_w)).unwrap();
				result.push(shrunk_graph);
			});
		});

		// Shrink by removing an edge
		if limits.iter().all(|l| l != &Limit::EdgeRemove)
		{
			for e in self.all_edges()
			{
				// Add to the result a copy of the graph
				// without the edge
				let mut shrunk_graph = self.clone();
				shrunk_graph.remove_edge(e).unwrap();
				result.push(shrunk_graph);
			}
		}

		// Shrink by removing a vertex that has no edges.
		// We don't remove any edges in this step (to be able to remove a vertex)
		// because we are already shrinking by removing edges, which means, there
		// should be a set of edge shrinkages that result in a removable vertex.
		if Limit::min_vertices(&limits) < self.all_vertices().count() {
			for v in self
				.all_vertices()
				// Dont touch untouchable vertices
				.filter(|&v| !limits.contains(&Limit::VertexKeep(v)))
			{
				if self.edges_incident_on(v).next().is_none()
				{
					let mut shrunk_graph = self.clone();
					shrunk_graph.remove_vertex(v).unwrap();
					result.push(shrunk_graph);
				}
			}
		}
		
		Box::new(result.into_iter())
	}
}

impl<D: Directedness> Arbitrary for MockGraph<D>
{
	fn arbitrary<G: Gen>(g: &mut G) -> Self
	{
		Self::arbitrary_guided(g, .., ..)
	}

	fn shrink(&self) -> Box<dyn Iterator<Item = Self>>
	{
		self.shrink_guided(HashSet::new())
	}
}

impl<D: Directedness> MockGraph<D>
{
	/// Performs Depth First Search recursively tracking which vertices have
	/// been visited in the 'visited' argument.
	///
	/// If given, will stop when the 'end' is visited.
	/// Returns whether 'end' was visited and false if 'end' isn't given or
	/// wasn't visited.
	pub fn dfs_rec(
		&self,
		start: MockVertex,
		end: Option<MockVertex>,
		visited: &mut Vec<MockVertex>,
	) -> bool
	{
		if let Some(end) = end
		{
			if start == end
			{
				return true;
			}
		}
		visited.push(start);
		if D::directed()
		{
			for e in self
				.edges_incident_on(start)
				.filter(|e| e.source() == start)
			{
				if !visited.contains(&e.sink())
				{
					if self.dfs_rec(e.sink(), end, visited)
					{
						return true; // early return of 'end' is found
					}
				}
			}
		}
		else
		{
			for e in self.edges_incident_on(start)
			{
				let v_other = if e.source() == start
				{
					e.sink()
				}
				else
				{
					e.source()
				};
				if !visited.contains(&v_other)
				{
					self.dfs_rec(v_other, end, visited);
				}
			}
		}
		false
	}

	/// Shrinks the graph by removing an edge, as long as the given closure
	/// succeeds on the resulting graph.
	///
	/// Adds all the shrunk graphs into the given vec.
	pub fn shrink_by_removing_edge<F>(&self, limits: &HashSet<Limit>, result: &mut Vec<Self>, f: F)
	where
		F: Fn(&Self) -> bool,
	{
		if !limits.contains(&Limit::EdgeRemove)
			&& !limits.contains(&Limit::EdgeMin(self.all_edges().count()))
		{
			result.extend(
				self.all_edges()
					.map(|e| {
						let mut g = self.clone();
						g.remove_edge_where_weight(e, |w| w == e.weight()).unwrap();
						g
					})
					.filter(|g| f(&g)),
			);
		}
	}

	/// Shrinks the graph by removing a vertex and replacing it with edges, such
	/// that all paths that went through the removed vertex are still there.
	///
	/// Adds all the shrunk graphs into the given vec.
	pub fn shrink_by_replacing_vertex_with_edges(
		&self,
		limits: &HashSet<Limit>,
		result: &mut Vec<Self>,
	)
	{
		if !limits.contains(&Limit::VertexRemove)
			&& !limits.contains(&Limit::EdgeRemove)
			&& !limits.contains(&Limit::VertexMin(self.all_vertices().count()))
		{
			for v in self
				.all_vertices()
				.filter(|v| !limits.contains(&Limit::VertexKeep(*v)))
			{
				let mut clone = self.clone();
				clone.remove_vertex(v).unwrap();
				if let Ok(g) = DirectedGraph::constrain_single(self)
				{
					for (sink, w1) in g.edges_sourced_in(v).map(|e| (e.sink(), e.weight_owned()))
					{
						if sink == v
						{
							continue;
						}
						for (source, w2) in
							g.edges_sinked_in(v).map(|e| (e.source(), e.weight_owned()))
						{
							if source == v
							{
								continue;
							}
							clone.add_edge_weighted((source, sink, w1.clone())).unwrap();
							clone.add_edge_weighted((source, sink, w2.clone())).unwrap();
						}
					}
				}
				else
				{
					let neighbors: Vec<_> = self
						.edges_incident_on(v)
						.map(|e| (e.other(v), e.weight_owned()))
						.collect();
					let mut neighbor_iter = neighbors.iter();
					while let Some(&(v1, w1)) = neighbor_iter.next()
					{
						if v1 == v
						{
							continue;
						}
						let rest = neighbor_iter.clone();
						for &(v2, w2) in rest
						{
							if v2 == v
							{
								continue;
							}
							clone.add_edge_weighted((v1, v2, w1.clone())).unwrap();
							clone.add_edge_weighted((v1, v2, w2.clone())).unwrap();
						}
					}
				}
				result.push(clone);
			}
		}
	}

	/// Shrinks the graph values, not removing any vertices or edges
	pub fn shrink_values(&self, limits: &HashSet<Limit>, result: &mut Vec<Self>)
	{
		let mut limits = limits.clone();
		limits.insert(Limit::EdgeRemove);
		limits.insert(Limit::VertexRemove);
		result.extend(self.clone().shrink_guided(limits));
	}
}
