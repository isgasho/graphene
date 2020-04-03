//! Tests the `core::Unilateral` trait and its ensurer
//! `core::UnilateralGraph`.
//!

use crate::mock_graph::{
	arbitrary::{
		ArbConnectedGraph, ArbNonUnilatralGraph, ArbTwoVerticesIn, ArbUnilatralGraph, ArbVertexIn,
		ArbVerticesIn,
	},
	MockEdgeWeight, MockVertexWeight,
};
use graphene::core::{
	property::{AddEdge, NewVertex, NonNull, RemoveEdge, RemoveVertex, UnilateralGraph},
	Directed, Edge, Ensure, Release,
};

/// Tests that UnilateralGraph correctly identifies unilateral graphs.
#[quickcheck]
fn accept_unilateral(g: ArbUnilatralGraph) -> bool
{
	UnilateralGraph::validate(&g.0.release_all())
}

/// Tests that UnilateralGraph correctly rejects non-unilateral graphs.
#[quickcheck]
fn reject_unilateral(g: ArbNonUnilatralGraph) -> bool
{
	!UnilateralGraph::validate(&g.0)
}

/// Tests that UnilateralGraph always accepts adding an edge
#[quickcheck]
fn accept_add_edge_weighted(
	mut g: ArbTwoVerticesIn<ArbUnilatralGraph>,
	e_weight: MockEdgeWeight,
) -> bool
{
	let (v1, v2) = g.get_both();
	g.add_edge_weighted((v1, v2, e_weight)).is_ok()
}

/// Tests that a UnilateralGraph accepts removing an edge that isn't critical
/// for unilateralism.
#[quickcheck]
fn accept_remove_edge_where(
	mut g: ArbTwoVerticesIn<ArbUnilatralGraph>,
	e_weight: MockEdgeWeight,
) -> bool
{
	let (v1, v2) = g.get_both();
	// To ensure we can remove an edge, we first create an edge to remove
	g.add_edge_weighted((v1, v2, e_weight.clone())).unwrap();

	g.remove_edge_where(|e| (e.source() == v1 && e.sink() == v2))
		.is_ok()
}

/// Tests that a UnilateralGraph rejects removing an edge that is critical for
/// unilateralism
///
/// TODO: Needs better graph generation. Right now uses 2 connected graphs,
/// should TODO: find a way to use unilateral graphs while still being able to
/// find a critical edge.
#[quickcheck]
fn reject_remove_edge_where(
	g1: ArbVertexIn<ArbConnectedGraph<Directed>>,
	g2: ArbVertexIn<ArbConnectedGraph<Directed>>,
	e_weight: MockEdgeWeight,
) -> bool
{
	let v1 = g1.get_vertex();
	let v2 = g2.get_vertex();
	let mut graph = g1.release_all();
	// We start by joining 2 connected graphs into a unconnected graph with the 2
	// components
	let v_map = graph.join(&g2);

	// We then connect the two components with 1 edge, making in unilateral.
	graph
		.add_edge_weighted((v1, v_map[&v2], e_weight.clone()))
		.unwrap();

	let mut unilateral = UnilateralGraph::ensure(graph).unwrap();

	// We now try to remove the the added edge
	unilateral
		.remove_edge_where(|e| (e.source() == v1 && e.sink() == v_map[&v2]))
		.is_err()
}

/// Tests that a UnilateralGraph accepts removing a vertex if the remaining
/// graph is still unilateral.
#[quickcheck]
fn accept_remove_vertex(
	mock: ArbVerticesIn<ArbTwoVerticesIn<ArbUnilatralGraph>>,
	v_weight: MockVertexWeight,
	e_weight: MockEdgeWeight,
) -> bool
{
	let v_set = mock.1;
	let two_v_in = mock.0.release();
	let (v1, v2) = two_v_in.get_both();
	let mut graph = (two_v_in.0).0.release_all();

	// It is only acceptable to remove a vertex (and any edge incident on it)
	// if after doing so, the rest of the vertices are still unilateral.

	// We take a unilateral graph and add new vertex to it.
	let v_new = graph.new_vertex_weighted(v_weight).unwrap();

	// We then connect it to the other vertices, making the whole graph unilaretal
	// again
	graph
		.add_edge_weighted((v_new, v1, e_weight.clone()))
		.unwrap();
	graph
		.add_edge_weighted((v2, v_new, e_weight.clone()))
		.unwrap();

	// We add auxiliary edges from the new vertex to the others
	for (idx, v_other) in v_set.into_iter().enumerate()
	{
		// just to add some variance
		if idx % 2 == 0
		{
			graph
				.add_edge_weighted((v_other, v_new, e_weight.clone()))
				.unwrap();
		}
		else
		{
			graph
				.add_edge_weighted((v_new, v_other, e_weight.clone()))
				.unwrap();
		}
	}

	// We then try to remove the vertex again
	UnilateralGraph::ensure_unvalidated(graph)
		.remove_vertex(v_new)
		.is_ok()
}

/// Tests that a UnilateralGraph rejects removing a vertex if it renders the
/// graph non-unilateral
///
/// TODO: Needs better graph generation. Right now uses 2 connected graphs,
/// should TODO: find a way to use unilateral graphs while still being able to
/// create a critical vertex.
#[quickcheck]
fn reject_remove_vertex(
	g1: ArbVertexIn<ArbConnectedGraph<Directed>>,
	g2: ArbVertexIn<ArbConnectedGraph<Directed>>,
	e_weight: MockEdgeWeight,
	v_weight: MockVertexWeight,
) -> bool
{
	let v1 = g1.get_vertex();
	let v2 = g2.get_vertex();
	let mut graph = g1.release_all();
	// We start by joining 2 connected graphs into a unconnected graph with the 2
	// components
	let v_map = graph.join(&g2);

	// We then connect the two components through a vertex, making it unilateral
	let new_v = graph.new_vertex_weighted(v_weight.clone()).unwrap();
	graph
		.add_edge_weighted((v1, new_v, e_weight.clone()))
		.unwrap();
	graph
		.add_edge_weighted((new_v, v_map[&v2], e_weight.clone()))
		.unwrap();

	let mut unilateral = UnilateralGraph::ensure(graph).unwrap();

	// We now try to remove the the added vertex
	unilateral.remove_vertex(new_v).is_err()
}
