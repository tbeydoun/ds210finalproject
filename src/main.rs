mod graph;
use graph::{analyze_top_movies, betweenness_centrality, closeness_centrality};
use graph::{create_graph, find_largest_component, read_genres_ratings, read_movies};
use petgraph::algo::{kosaraju_scc};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

fn main() {
    let movies_result = read_movies("/Users/talyabeydoun/proj/src/movies.csv"); //Path will need to be changed

    let mut movie_id_to_genres = HashMap::new();
    if let Ok(ref movies) = movies_result {
    for movie in movies {
        movie_id_to_genres.insert(movie.movie_id, movie.genres.clone());
    }
    }
    let movies = movies_result.expect("Failed to read movies");
         
    let genre_ratings = read_genres_ratings("/Users/talyabeydoun/proj/src/ratings.csv", &movie_id_to_genres); //Path will need to be changed

    let genre_ratings = genre_ratings.unwrap();
    let graph = create_graph(movies, genre_ratings);

    println!("Number of nodes in the graph: {}", graph.node_count());
    println!("Number of edges in the graph: {}", graph.edge_count());

    let connected_components = kosaraju_scc(&graph);
    let largest_component_index = find_largest_component(&connected_components);

    let betweenness_centrality = betweenness_centrality(&graph);
    let closeness_centrality = closeness_centrality(&graph, &connected_components[largest_component_index]);

    let top_n = 10;
    analyze_top_movies(&graph, &betweenness_centrality, &closeness_centrality, top_n);
}