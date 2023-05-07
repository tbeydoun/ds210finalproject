use petgraph::graph::NodeIndex;
use petgraph::Undirected;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::collections::VecDeque;
use petgraph::algo::dijkstra;
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use ordered_float::OrderedFloat;
use petgraph::Graph;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct Movie {
    #[serde(rename= "movieId")]
    pub movie_id: i32,
    #[serde(rename = "title")]
    pub title: String,
    #[serde(rename = "genres", deserialize_with = "deserialize_genres")]
    pub genres: Vec<String>,
}

fn deserialize_genres<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer).map_err(serde::de::Error::custom)?;
    let genres: Vec<String> = s.split('|').map(|genres| genres.to_string()).collect();
    Ok(genres)
}

#[derive(Debug, Deserialize)]
pub struct Rating {
    #[serde(rename= "userId")]
    pub user_id: i32,
    #[serde(rename= "movieId")]
    pub movie_id: i32,
    pub rating: f64,
    pub timestamp: i64,
}

fn get_genres_key(genres: &Vec<String>) -> String {
    let mut genres_key = genres.clone();
    genres_key.sort();
    genres_key.join(",")
}

pub fn create_graph(
    movies: Vec<Movie>,
    genres_ratings: HashMap<String, f64>,
) -> Graph<Movie, f64, Undirected> {
    let mut graph = Graph::<Movie, f64, Undirected>::new_undirected();

    // Add movie nodes to the graph
    for movie in movies.iter() {
        graph.add_node(movie.clone());
    }

    // Calculate edge weights and add edges to the graph
    for (i, movie1) in movies.iter().enumerate() {
        for movie2 in &movies[i + 1..] {
            let genres_key = get_genres_key(&movie1.genres);
            let edge_weight = genres_ratings.get(&genres_key).unwrap_or(&0.0);

            if *edge_weight > 0.0 {
                let node1 = NodeIndex::new(movie1.movie_id as usize);
                let node2 = NodeIndex::new(movie2.movie_id as usize);

                if graph.node_weight(node1).is_some() && graph.node_weight(node2).is_some() {                   
                graph.add_edge(node1, node2, *edge_weight);
                }
            }
        }
    }
    graph
}

pub fn read_genres_ratings(file_path: &str, movie_id_to_genres: &HashMap<i32, Vec<String>>) -> Result<HashMap<String, f64>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b',')
        .trim(csv::Trim::All)
        .from_reader(file);

    let ratings: Vec<Rating> = rdr
        .deserialize()
        .map(|result| result.unwrap())
        .collect();

    let mut genre_ratings = HashMap::new();

    for rating in ratings {
        if let Some(genres) = movie_id_to_genres.get(&rating.movie_id) {
            for genre in genres {
                let entry = genre_ratings.entry(genre.to_owned()).or_insert(0.0);
                *entry += rating.rating;
            }
        }
    }

    Ok(genre_ratings)
}

pub fn read_movies(file_path: &str) -> Result<Vec<Movie>> {
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut movies: Vec<Movie> = Vec::new();

    for result in rdr.deserialize() {
        let record: Movie = result?;
        movies.push(record);
    }

    Ok(movies)
}

pub fn betweenness_centrality(graph: &Graph<Movie, f64, Undirected>) -> Vec<f64> {
    let mut betweenness = vec![0.0; graph.node_count()];
    let mut stack = Vec::new();
    let mut sigma = vec![0.0; graph.node_count()];
    let mut distance = vec![0; graph.node_count()];
    let mut predecessors: Vec<Vec<NodeIndex>> = Vec::new();
    let mut delta = vec![0.0; graph.node_count()];

    for node in graph.node_indices() {
        predecessors.resize(graph.node_count(), Vec::new());
        for i in graph.node_indices() {
            sigma[i.index()] = 0.0;
            distance[i.index()] = -1;
        }

        sigma[node.index()] = 1.0;
        distance[node.index()] = 0;
        stack.clear();

        let mut queue = VecDeque::new();
        queue.push_back(node);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for neighbor in graph.neighbors(v) {
                if distance[neighbor.index()] < 0 {
                    queue.push_back(neighbor);
                    distance[neighbor.index()] = distance[v.index()] + 1;
                }

                if distance[neighbor.index()] == distance[v.index()] + 1 {
                    sigma[neighbor.index()] += sigma[v.index()];
                    predecessors[neighbor.index()].push(v);
                }
            }
        }

        for i in graph.node_indices() {
            delta[i.index()] = 0.0;
        }

        while let Some(w) = stack.pop() {
            for v in &predecessors[w.index()] {
                delta[v.index()] += (sigma[v.index()] / sigma[w.index()]) * (1.0 + delta[w.index()]);
            }
            if w != node {
                betweenness[w.index()] += delta[w.index()];
            }
        }
    }

    betweenness
}

pub fn closeness_centrality(
    graph: &Graph<Movie, f64, Undirected>,
    largest_component: &[NodeIndex],
) -> HashMap<NodeIndex, f64> {
    let n = graph.node_count();
    let mut closeness_centrality = HashMap::new();

    for node in largest_component {
        let paths = dijkstra(&graph, *node, Some(*node), |edge| *edge.weight());
        let sum_distances: f64 = paths.values().sum();
        if sum_distances > 0.0 {
            let closeness = (n as f64 - 1.0) / sum_distances;
            closeness_centrality.insert(*node, closeness);
        }
    }
    closeness_centrality
}

pub fn analyze_top_movies(
    graph: &Graph<Movie, f64, Undirected>,
    betweenness_centrality: &[f64],
    closeness_centrality: &HashMap<NodeIndex, f64>,
    top_n: usize,
) {    
    let mut top_betweenness = BinaryHeap::with_capacity(top_n);
    let mut top_closeness = BinaryHeap::with_capacity(top_n);

    for node in graph.node_indices() {
        let bc = betweenness_centrality[node.index()];
        let cc = closeness_centrality.get(&node).unwrap_or(&0.0);

        if top_betweenness.len() < top_n {
            top_betweenness.push(Reverse((OrderedFloat(bc), node)));
        } else if let Some(&Reverse((smallest_bc, _))) = top_betweenness.peek() {
            if bc > *smallest_bc {
                top_betweenness.pop();
                top_betweenness.push(Reverse((OrderedFloat(bc), node)));
            }
        }

        if top_closeness.len() < top_n {
            top_closeness.push(Reverse((OrderedFloat(*cc), node)));
        } else if let Some(&Reverse((smallest_cc, _))) = top_closeness.peek() {
            if *cc > *smallest_cc {
                top_closeness.pop();
                top_closeness.push(Reverse((OrderedFloat(*cc), node)));
            }
        }
    }

    println!("Top {} movies by betweenness centrality:", top_n);
    let mut top_movies = top_betweenness.into_sorted_vec();
    for (i, Reverse((score, node))) in top_movies.iter().enumerate() {
        let movie = graph.node_weight(*node).unwrap();
        println!(
            "{}. {} - Genres: {:?}, betweenness centrality: {}",
            i + 1,
            movie.title,
            movie.genres,
            score
        );
    }

    println!("\nTop {} movies by closeness centrality:", top_n);
    top_movies = top_closeness.into_sorted_vec();
    for (i, Reverse((score, node))) in top_movies.iter().enumerate() {
        let movie = graph.node_weight(*node).unwrap();
        println!(
            "{}. {} - Genres: {:?}, closeness centrality: {}",
            i + 1,
            movie.title,
            movie.genres,
            score
        );
    }
}

pub fn find_largest_component(components: &Vec<Vec<NodeIndex>>) -> usize {
    let mut largest_component_index = 0;
    let mut max_size = 0;

    for (index, component) in components.iter().enumerate() {
        if component.len() > max_size {
            max_size = component.len();
            largest_component_index = index;
        }
    }
    largest_component_index
}