use crate::mock_graph::{
	arbitrary::{ArbTwoVerticesIn, GuidedArbGraph, Limit, NonUnique},
	MockEdgeWeight, MockVertex, MockVertexWeight,
};
use graphene::{
	core::{
		property::{NonNull, VertexInGraph},
		Ensure, Graph, GraphDeref, GraphDerefMut, Release,
	},
	impl_ensurer,
};
use quickcheck::{Arbitrary, Gen};
use std::{
	collections::{hash_map::RandomState, HashSet},
	ops::RangeBounds,
};

/// An arbitrary graph and a vertex in it.
///
/// Note: All graphs will have at least 1 vertex, meaning this type never
/// includes the empty graph.
#[derive(Clone, Debug)]
pub struct ArbVertexIn<G>(pub VertexInGraph<G>)
where
	G: GuidedArbGraph + Ensure + Clone,
	G::Graph: Clone
		+ Graph<Vertex = MockVertex, VertexWeight = MockVertexWeight, EdgeWeight = MockEdgeWeight>;
impl<Gr> Arbitrary for ArbVertexIn<Gr>
where
	Gr: GuidedArbGraph + Ensure + Clone + GraphDerefMut,
	Gr::Graph: Clone
		+ Graph<Vertex = MockVertex, VertexWeight = MockVertexWeight, EdgeWeight = MockEdgeWeight>,
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
impl<Gr> GuidedArbGraph for ArbVertexIn<Gr>
where
	Gr: GuidedArbGraph + Ensure + Clone + GraphDerefMut,
	Gr::Graph: Clone
		+ Graph<Vertex = MockVertex, VertexWeight = MockVertexWeight, EdgeWeight = MockEdgeWeight>,
{
	fn arbitrary_guided<G: Gen>(
		g: &mut G,
		v_range: impl RangeBounds<usize>,
		e_range: impl RangeBounds<usize>,
	) -> Self
	{
		let arb = ArbTwoVerticesIn::<_, NonUnique>::arbitrary_guided(g, v_range, e_range);
		Self(VertexInGraph::new_unvalidated(arb.0, arb.1))
	}

	fn shrink_guided(&self, limits: HashSet<Limit, RandomState>) -> Box<dyn Iterator<Item = Self>>
	{
		Box::new(
			ArbTwoVerticesIn::new(
				self.0.clone().release(),
				self.get_vertex(),
				self.get_vertex(),
			)
			.shrink_guided(limits)
			.map(|ArbTwoVerticesIn::<_, NonUnique>(g, v, _, _)| {
				Self(VertexInGraph::new_unvalidated(g, v))
			}),
		)
	}
}

impl_ensurer! {
	use<G> ArbVertexIn<G>:
	// Can never impl the following because MockGraph doesn't
	Reflexive
	as ( self.0) : VertexInGraph<G>
	where
	G: GuidedArbGraph + Ensure + Clone,
	G::Graph: Clone +
		Graph<Vertex = MockVertex, VertexWeight = MockVertexWeight, EdgeWeight = MockEdgeWeight>
}
