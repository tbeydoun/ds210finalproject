use super::graph;
use petgraph::Graph;
use petgraph::Undirected;
use std::collections::HashMap;

#[test]
fn test_read_movies() {
    let file_path = "src/movies.csv";
    let movies = graph::read_movies(file_path).unwrap();
    assert!(!movies.is_empty());
}

#[test]
fn test_create_graph() {
    let movies = vec![];
    let genres_ratings = HashMap::new();
    let graph: Graph<graph::Movie, f64, Undirected> = graph::create_graph(movies, genres_ratings);
    assert!(graph.node_count() == 0);
    assert!(graph.edge_count() == 0);
}

#[test]
fn test_betweenness_centrality() {
    let graph: Graph<graph::Movie, f64, Undirected> = Graph::new_undirected();
    let betweenness = graph::betweenness_centrality(&graph);
    assert!(betweenness.is_empty());
}

#[test]
fn test_closeness_centrality() {
    let graph: Graph<graph::Movie, f64, Undirected> = Graph::new_undirected();
    let largest_component = vec![];
    let closeness = graph::closeness_centrality(&graph, &largest_component);
    assert!(closeness.is_empty());
}

#[test]
fn test_find_largest_component() {
    let components = vec![vec![], vec![]];
    let largest_component_index = graph::find_largest_component(&components);
    assert_eq!(largest_component_index, 0);
}