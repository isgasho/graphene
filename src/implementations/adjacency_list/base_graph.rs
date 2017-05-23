use implementations::adjacency_list::*;


impl<T> AdjListGraph<T> {
	
	pub fn new(values: Vec<T>, edges: Vec<(usize, usize)>) -> Option<AdjListGraph<T>> {
		let mut g = AdjListGraph{ edges: Vec::new(), values: values };
		
		//Validate all edges point to vertices
		for &(source, sink) in &edges {
			if source >= g.values.len() || sink >= g.values.len(){
				return None;
			}
		}
		
		//Initialise adjacency list
		for _ in 0..g.values.len(){
			g.edges.push(Vec::new());
		}
		
		//Insert each edge
		for &(source, sink) in &edges {
			g.edges[source].push(sink);
		}
		Some(g)
	}
	
	
}
