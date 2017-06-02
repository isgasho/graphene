use common::adjacency_list::*;
use core::*;



impl<'a, T> BaseGraph<'a>
for AdjListGraph<T>
	where
		T: Copy + Eq
{
	type Vertex = T;
	type Weight = ();
	type VertexIter = Vec<T>;
	type EdgeIter = Vec<BaseEdge<T,()>>;
	
	
	fn all_vertices(&'a self) -> Vec<T> {
		let mut result = Vec::new();
		
		//For each value, output a copy
		for i in 0..self.values.len() {
			result.push(self.values[i]);
		}
		result
	}
	
	fn all_edges(&'a self) -> Vec<BaseEdge<T,()>> {
		let mut result = Vec::new();
		
		//For each vertex (source)
		for (source_i, ref out) in self.edges.iter().enumerate() {
			let source_value = self.values[source_i];
			//For each outgoing edge (sink)
			for &sink_i in out.iter() {
				let sink_value = self.values[sink_i];
				//Return the edge
				result.push(BaseEdge::new(source_value, sink_value,()));
			}
		}
		result
	}
	
	fn add_vertex(&mut self, v: T) -> Result<(),()>{
		
		if self.values.contains(&v){
			Err(())
		}else{
			self.values.push(v);
			self.edges.push(Vec::new());
			Ok(())
		}
	}
	
	fn remove_vertex(&mut self, v: T) -> Result<(),()>{
		//Get index of vertex
		if let Some(v_i) = self.get_index(v){
			/* Remove all incoming edges to v
			 */
			//Go through all vertices
			for t_v_i in 0..self.values.len(){
				let mut to_remove = Vec::new();
				//Go through all outgoing edges
				let ref mut t_v_out = self.edges[t_v_i];
				for i in 0..t_v_out.len() {
					//If an edge points to v, collect its index
					if t_v_out[i] == v_i {
						to_remove.push(i);
					}
				}
				//Delete all collected edges
				for i in (0..to_remove.len()).rev() {
					//Delete the last indices first so
					//that the other indices aren't invalidated
					t_v_out.remove(to_remove[i]);
				}
			}
			
			/* Re-point all edges pointing to last value (last)
			 * to point to v
			 */
			let last_i = self.values.len() - 1;
			//For each vertex
			for t_v_i in 0..self.edges.len() {
				//any edge pointing to the old last value
				//should now point to v
				let ref mut t_v_out = self.edges[t_v_i];
				for edge_i in 0..t_v_out.len(){
					if t_v_out[edge_i] == last_i {
						t_v_out[edge_i] = v_i
					}
				}
			}
			
			/*Remove v, swapping in the value of last
			 */
			self.values.swap_remove(v_i);
			self.edges.swap_remove(v_i);
			return Ok(());
		}
		//Vertex not part of the core
		Err(())
	}
	
	fn add_edge(&mut self, e: BaseEdge<T,()>) -> Result<(), ()>{
		self.if_valid_edge( e, |s, source_i, sink_i|{
			s.edges[source_i].push(sink_i);
			Ok(())
		})
	}
	
	fn remove_edge(&mut self, e: BaseEdge<T,()>)-> Result<(), ()>{
		self.if_valid_edge(e, |s, source_i, sink_i|{
			if let Some(i) = s.edges[source_i].iter().position(|&sink_cand| sink_cand == sink_i) {
				s.edges[source_i].remove(i);
				return Ok(());
			}
			Err(())
		})
	}
}













