use super::*;

enum Operations<V,W>
	where
		V: Vertex,
		W: Weight,
{
	AddVertex(V),
	AddEdge(BaseEdge<V,W>),
	RemoveVertex(V),
	RemoveEdge(BaseEdge<V,W>),
}

use self::Operations::*;

pub struct Unconstrainer<'a,V,W,Vi,Ei,G>
	where
		V: Vertex,
		W: Weight,
		Vi: VertexIter<V>,
		Ei: EdgeIter<V,W>,
		G: 'a + ConstrainedGraph<Vertex=V,Weight=W,VertexIter=Vi,EdgeIter=Ei>
{
	graph: &'a mut G,
	operations: Vec<Operations<V,W>>,
}

impl<'a,V,W,Vi,Ei,G> Unconstrainer<'a,V,W,Vi,Ei,G>
	where
		V: Vertex,
		W: Weight,
		Vi: VertexIter<V>,
		Ei: EdgeIter<V,W>,
		G: ConstrainedGraph<Vertex=V,Weight=W,VertexIter=Vi,EdgeIter=Ei>
{
	
	pub fn new(g: &'a mut G) -> Self{
		Unconstrainer{graph:g, operations: Vec::new()}
	}
	
	pub fn add_vertex(mut self, v: V) -> Self{
		self.operations.push(Operations::AddVertex(v));
		self
	}
	
	pub fn remove_vertex(mut self, v:V) -> Self{
		self.operations.push(Operations::RemoveVertex(v));
		self
	}
	
	pub fn add_edge(mut self, e: BaseEdge<V,W>) -> Self{
		self.operations.push(Operations::AddEdge(e));
		self
	}
	
	pub fn remove_edge(mut self, e: BaseEdge<V,W>) -> Self{
		self.operations.push(Operations::RemoveEdge(e));
		self
	}
	
	pub fn constrain(mut self) -> Result<(),()> {
		match self.execute_unconstrained_operations(){
			Err(i) =>{
				// One of the operations failed, therefore roll back changes
				self.rollback_operations(i);
				Err(())
			}
			Ok(()) =>{
				// All operations accepted, test invariant
				if self.graph.invariant_holds() {
					Ok(())
				}else{
					let op_count = self.operations.len();
					self.rollback_operations(op_count);
					Err(())
				}
			}
		}
	}
	
	fn rollback_operations(&mut self, i:usize) {
		let ref operations = self.operations;
		let ref mut graph = self.graph;
		
		for j in (0..(i+1)).rev(){
			unsafe{
				match operations[j] {
					AddVertex(v) => graph.uncon_remove_vertex(v),
					AddEdge(e) => graph.uncon_remove_edge(e),
					RemoveVertex(v) => graph.uncon_add_vertex(v),
					RemoveEdge(e) => graph.uncon_add_edge(e),
				}.unwrap()
			}
		}
	}
	
	fn execute_unconstrained_operations(&mut self) -> Result<(),usize>{
		let ref operations = self.operations;
		let ref mut graph = self.graph;
		
		let mut i = 0;
		while i < operations.len() {
			match unsafe {
				match operations[i] {
					AddVertex(v) => graph.uncon_add_vertex(v),
					AddEdge(e) => graph.uncon_add_edge(e),
					RemoveVertex(v) => graph.uncon_remove_vertex(v),
					RemoveEdge(e) => graph.uncon_remove_edge(e),
				}
			}{
				Err(()) =>{
					// Failed at operation i
					return Err(i);
				}
				Ok(())	=> i += 1,
			}
		}
		Ok(())
	}
	
}


pub trait ConstrainedGraph: BaseGraph
where
	Self: Sized,
{
	fn invariant_holds(&self) -> bool;
	
	unsafe fn uncon_add_vertex(&mut self, v: Self::Vertex) -> Result<(),()>;
	
	unsafe fn uncon_remove_vertex(&mut self, v: Self::Vertex) -> Result<(),()>;
	
	unsafe fn uncon_add_edge(&mut self, e: BaseEdge<Self::Vertex,Self::Weight>) -> Result<(),()>;
	
	unsafe fn uncon_remove_edge(&mut self, e: BaseEdge<Self::Vertex,Self::Weight>) -> Result<(),()>;
	
	fn unconstrained<'a>(&'a mut self) -> Unconstrainer<
		Self::Vertex, Self::Weight, Self::VertexIter, Self::EdgeIter, Self>{
		Unconstrainer::new(self)
	}
	
	
}

