///
/// Tests `TarjanSCC`: Tarjan's algorithm for finding strongly connected components.
///

use graphene::{
	algo::{TarjanSCC},
	core::{
		Directed, Graph, Constrainer,
		constraint::{ConnectedGraph, Subgraph},
	},
};
use crate::mock_graph::{
	MockGraph,
	arbitrary::{ArbVertexIn},
};

///
/// Tests that no produced SCC is empty
///
#[quickcheck]
fn produces_non_empty_components(ArbVertexIn(graph, v): ArbVertexIn<MockGraph<Directed>>) -> bool
{
	for scc in TarjanSCC::new(&graph, v) {
		if scc.all_vertices().count() == 0 {
			return false;
		}
	}
	true
}

///
/// Tests that any SCC returned is actually strongly connected.
///
#[quickcheck]
fn produces_connected_components(ArbVertexIn(graph, v): ArbVertexIn<MockGraph<Directed>>) -> bool
{
	for scc in TarjanSCC::new(&graph, v) {
		if ConnectedGraph::constrain_single(scc).is_err() {
			return false
		}
	}
	true
}

///
/// Tests that for any SCC pair produced, they are not strongly connected.
///
#[quickcheck]
fn produces_disconnected_components(ArbVertexIn(graph, v): ArbVertexIn<MockGraph<Directed>>) -> bool
{
	let sccs = TarjanSCC::new(&graph, v).collect::<Vec<_>>();
	let mut scc_iter = sccs.iter();
	
	while let Some(scc) = scc_iter.next() {
		for scc2 in scc_iter.clone() {
			if scc.reaches(scc2).is_some() {
				if scc2.reaches(scc).is_some() {
					return false;
				}
			}
		}
	}
	true
}

///
/// Tests that all vertices are put inside some produced SCC.
///
#[quickcheck]
fn produces_all_vertices(ArbVertexIn(graph, v): ArbVertexIn<MockGraph<Directed>>) -> bool
{
	// We simply count the vertices since we have another test
	// for checking that no vertex is reused
	let mut vertex_count = 0;
	for scc in TarjanSCC::new(&graph, v) {
		vertex_count += scc.all_vertices().count();
	}
	vertex_count == graph.all_vertices().count()
}

///
/// Tests that all vertices in the components are from the original graph.
///
#[quickcheck]
fn produces_only_valid_vertices(ArbVertexIn(graph, v): ArbVertexIn<MockGraph<Directed>>) -> bool
{
	for scc in TarjanSCC::new(&graph, v) {
		for v in scc.all_vertices() {
			if !graph.contains_vertex(v) {
				return false;
			}
		}
	}
	true
}

///
/// Tests that no two produced SCCs share any vertices
///
#[quickcheck]
fn produces_vertex_disjoint_components(ArbVertexIn(graph, v): ArbVertexIn<MockGraph<Directed>>)
	-> bool
{
	let sccs = TarjanSCC::new(&graph, v).collect::<Vec<_>>();
	let mut scc_iter = sccs.iter();
	
	while let Some(scc) = scc_iter.next() {
		for scc2 in scc_iter.clone() {
			for v in scc.all_vertices() {
				if scc2.contains_vertex(v) {
					return false;
				}
			}
		}
	}
	true
}

///
/// Tests that the SCCs are produced in some reverse topological order.
/// This is a guarantee of Tarjan's algorithm, which means if we don't do that,
/// we are not implementing it correctly.
///
#[quickcheck]
fn produces_reverse_topological_ordering(ArbVertexIn(graph, v): ArbVertexIn<MockGraph<Directed>>) -> bool
{
	// To test the ordering, we simply check that an earlier-produced component can't reach any
	// later one.
	let sccs = TarjanSCC::new(&graph, v).collect::<Vec<_>>();
	let mut scc_iter = sccs.iter();
	
	while let Some(scc) = scc_iter.next() {
		for scc2 in scc_iter.clone() {
			if scc.reaches(scc2).is_some() {
				return false;
			}
		}
	}
	true
}