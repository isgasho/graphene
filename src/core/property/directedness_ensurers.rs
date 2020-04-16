use crate::core::{Directed, Directedness, Ensure, Graph, Undirected};
use delegate::delegate;

#[derive(Clone, Debug)]
pub struct DirectedGraph<C: Ensure>(C);

impl<C: Ensure> Ensure for DirectedGraph<C>
{
	fn ensure_unvalidated(c: Self::Ensured, _: ()) -> Self
	{
		Self(c)
	}

	fn validate(_: &Self::Ensured, _: &()) -> bool
	{
		<<C::Graph as Graph>::Directedness as Directedness>::directed()
	}
}

impl<C: Ensure> Graph for DirectedGraph<C>
{
	type Directedness = Directed;
	type EdgeWeight = <C::Graph as Graph>::EdgeWeight;
	type Vertex = <C::Graph as Graph>::Vertex;
	type VertexWeight = <C::Graph as Graph>::VertexWeight;

	delegate! {
		to self.0.graph() {
			fn all_vertices_weighted<'a>(
				&'a self,
			) -> Box<dyn 'a + Iterator<Item = (Self::Vertex, &'a Self::VertexWeight)>>;

			fn all_edges<'a>(
				&'a self,
			) -> Box<dyn 'a + Iterator<Item = (Self::Vertex, Self::Vertex, &'a Self::EdgeWeight)>>;
		}
	}
}

impl_ensurer! {
	use<C> DirectedGraph<C>: Ensure, Graph, DirectedConstraint
	as (self.0) : C
}

#[derive(Clone, Debug)]
pub struct UndirectedGraph<C: Ensure>(C);

impl<C: Ensure> Ensure for UndirectedGraph<C>
{
	fn ensure_unvalidated(c: Self::Ensured, _: ()) -> Self
	{
		Self(c)
	}

	fn validate(_: &Self::Ensured, _: &()) -> bool
	{
		!<<C::Graph as Graph>::Directedness as Directedness>::directed()
	}
}

impl<C: Ensure> Graph for UndirectedGraph<C>
{
	type Directedness = Undirected;
	type EdgeWeight = <C::Graph as Graph>::EdgeWeight;
	type Vertex = <C::Graph as Graph>::Vertex;
	type VertexWeight = <C::Graph as Graph>::VertexWeight;

	delegate! {
		to self.0.graph() {
			fn all_vertices_weighted<'a>(
				&'a self,
			) -> Box<dyn 'a + Iterator<Item = (Self::Vertex, &'a Self::VertexWeight)>>;

			fn all_edges<'a>(
				&'a self,
			) -> Box<dyn 'a + Iterator<Item = (Self::Vertex, Self::Vertex, &'a Self::EdgeWeight)>>;
		}
	}
}

impl_ensurer! {
	use<C> UndirectedGraph<C>: Ensure, Graph, UndirectedConstraint
	as (self.0) : C
}
