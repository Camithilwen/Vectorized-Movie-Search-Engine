use io::read_csv;
use qdrant_client::Qdrant;
use qdrant_client::QdrantError;
use pandrs::*;
use std::convert::Infallible;
fn main() -> Result<Infallible, QdrantError> {
    let client = Qdrant::from_url("https://93f18d10-87f0-43bb-9108-d1b0cfe91b13.us-east4-0.gcp.cloud.qdrant.io")
        .api_key(std::env::var("https://93f18d10-87f0-43bb-9108-d1b0cfe91b13.us-east4-0.gcp.cloud.qdrant.io"))
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
}

fn preprocessing() -> Result<DataFrame, PandRSError> {
    let raw_data = DataFrame::from_csv("data/wiki_movie_plots_deduped.csv", true)?;
    let mut mut_data = raw_data.clone();
    let mut_data("Release Year") = mut_data("Release Year") as &str;
}
