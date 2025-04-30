use qdrant_client::qdrant::{
    vectors_config::Config, Distance, PointId, PointStruct,
    VectorsConfig, VectorParams, CreateCollectionBuilder,
    MultiVectorConfig, MultiVectorComparator, DenseVector, Vector,
    vectors::VectorsOptions
};
use ::qdrant_client::Qdrant;
use polars::prelude::*;
use polars_lazy::prelude::*;
use std::collections::HashMap;
use embed_anything::{embeddings::embed::*, embed_query, embeddings::embed::EmbeddingResult};
use embed_anything::embeddings::local::text_embedding::ONNXModel;
use std::process::{Command, Stdio};
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Qdrant::from_url("https://93f18d10-87f0-43bb-9108-d1b0cfe91b13.us-east4-0.gcp.cloud.qdrant.io")
        .api_key(std::env::var("QDRANT_API_KEY"))
        .timeout(std::time::Duration::from_secs(10))
        .skip_compatibility_check()
        .build()?;

    //Preprocess data
    let processed_data = preprocessing()?;

    //Extract text data
    let movie_texts = extract_text_data(processed_data.clone())?;

    //Embed text
    let embeddings = embedder(&movie_texts).await?;

    //Create qdrant collection
    create_collection(&client, "movie_plots").await?;

    //Upload collection to qdrant cloud
    upload_vectors(&client, "movie_plots", processed_data, embeddings).await?;

    Ok(())
}

//Preprocess data
fn preprocessing() -> PolarsResult<LazyFrame> {
    //Read data
    let raw_data = csv_read()?;
    let data = raw_data.lazy();

    //Convert "Release Year" column front int to string.
    let mut target_columns = PlHashMap::new();
    target_columns.insert("Release Year", DataType::String);
    let processed_data = data.cast(target_columns, true);

    //Return data
    Ok(processed_data)
}

//Read in csv dataset
fn csv_read() -> PolarsResult<DataFrame> {
    CsvReadOptions::default()
            .with_has_header(true)
            .try_into_reader_with_file_path(Some("data/wiki_movie_plots_deduped.csv".into()))?
            .finish()
}

//Extract text from LazyFrame
fn extract_text_data(lf: LazyFrame) -> PolarsResult<Vec<String>> {
    //Combine title and plot columns into single text column
    let text_df = lf.select([
        col("Title"),
        col("Plot")
    ])
    .collect()?;
    let titles = text_df.column("Title")?.str()?;
    let plots = text_df.column("Plot")?.str()?;

    let mut texts = Vec::with_capacity(titles.len());
    for i in 0..titles.len() {
        let title = titles.get(i).unwrap_or_default();
        let plot = plots.get(i).unwrap_or_default();
        texts.push(format!("Title: {} Plot: {}", title, plot));
    }

    Ok(texts)
}

//vectorize dataset using ColBERT.
async fn embedder(texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
    let input = serde_json::to_string(texts)?;

    // Call the ColBERT model using the command line
    let mut child = Command::new("python3")
        .arg("src/jina_colbert.py")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python script failed: {}", stderr).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.starts_with("ERROR:") {
        return Err(stdout.to_string().into());
    }

    // This is the key fix - explicitly specify the return type
    let embeddings: Vec<Vec<f32>> = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse embeddings: {} (raw output: {})", e, stdout))?;

    Ok(embeddings)
}

//Create qdrant collection with multi-vector support
async fn create_collection(client: &Qdrant, collection_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    //Check for existing collection
    let collections = client.list_collections().await?;
    if collections.collections.iter().any(|c| c.name == collection_name) {
        println!("Collection '{}' already exists", collection_name);
        return Ok(());
    }

    // Create collection with multi-vector configuration
    // JinaColBERT outputs token-level embeddings, typically of dimension 128
    let collection_config = CreateCollectionBuilder::default()
        .collection_name(collection_name)
        .vectors_config(VectorsConfig {
            config: Some(Config::Params(VectorParams {
                size: 128, // JinaColBERT token vector dimension
                distance: Distance::Cosine.into(),
                hnsw_config: None, //can consider customizing HNSW parameters if needed
                quantization_config: None,
                on_disk: None,
                multivector_config: Some(MultiVectorConfig {
                    comparator: MultiVectorComparator::MaxSim.into(),
                }),
                ..Default::default()
            })),
        })
        .build();

    client.create_collection(collection_config).await?;
    println!("Created collection: {} with multi-vector support", collection_name);

    Ok(())
}

async fn upload_vectors(
    client: &Qdrant,
    collection_name: &str,
    data: LazyFrame,
    embeddings: Vec<Vec<f32>>,  // Changed from EmbedData to Vec<Vec<f32>>
) -> Result<(), Box<dyn std::error::Error>> {
    // Collect metadata
    let metadata_df = data.select([
        col("Title"),
        col("Release Year"),
        col("Origin/Ethnicity"),
        col("Director"),
        col("Genre")
    ]).collect()?;

    // Create upsert points
    let mut points = Vec::new();

    for (idx, embedding) in embeddings.iter().enumerate() {
        // Create payload from metadata
        let mut payload = HashMap::new();

        if let Ok(row) = metadata_df.get_row(idx) {
            if let Some(title) = row.0.get(0) {
                payload.insert("title".to_string(), title.to_string().into());
            }
            if let Some(year) = row.0.get(1) {
                payload.insert("year".to_string(), year.to_string().into());
            }
            if let Some(origin) = row.0.get(2) {
                payload.insert("origin".to_string(), origin.to_string().into());
            }
            if let Some(director) = row.0.get(3) {
                payload.insert("director".to_string(), director.to_string().into());
            }
            if let Some(genre) = row.0.get(4) {
                payload.insert("genre".to_string(), genre.to_string().into());
            }
        }

        // Convert embedding to Qdrant's MultiDenseVector format
        let dense_vectors = vec![DenseVector {
            data: embedding.clone(),
        }];

        let point = PointStruct {
            id: Some(PointId {
                point_id_options: Some(::qdrant_client::qdrant::point_id::PointIdOptions::Num(idx as u64)),
            }),
            vectors: Some(::qdrant_client::qdrant::Vectors {
                vectors_options: Some(VectorsOptions::Vector(
                    Vector::from(qdrant_client::qdrant::MultiDenseVector {
                        vectors: dense_vectors,
                    })
                )),
            }),
            payload,
        };

        points.push(point);

        // Batch upload in chunks of 100
        if points.len() >= 100 || idx == embeddings.len() - 1 {
            let upsert_request = qdrant_client::qdrant::UpsertPointsBuilder::new(
                collection_name,
                points.clone()
            ).build();

            client.upsert_points(upsert_request).await?;
            points.clear();
            println!("Uploaded batch ending at index {}", idx);
        }
    }

    Ok(())
}
