//! Unit tests for Embedding value object

use mcb_domain::Embedding;
use rstest::*;

#[rstest]
#[case(vec![0.1, 0.2, 0.3, 0.4, 0.5], "text-embedding-ada-002", 5)]
#[case(vec![0.1; 10], "text-embedding-3-small", 10)]
#[case(vec![0.0; 1536], "large-model", 1536)]
#[case(vec![], "empty-model", 0)]
fn test_embedding_creation_and_properties(
    #[case] vector: Vec<f32>,
    #[case] model: &str,
    #[case] dimensions: usize,
) {
    let embedding = Embedding {
        vector: vector.clone(),
        model: model.to_string(),
        dimensions,
    };

    assert_eq!(embedding.vector, vector);
    assert_eq!(embedding.model, model);
    assert_eq!(embedding.dimensions, dimensions);

    if !vector.is_empty() {
        assert_eq!(embedding.vector.len(), dimensions);
    } else {
        assert!(embedding.vector.is_empty());
        assert_eq!(embedding.dimensions, 0);
    }
}
