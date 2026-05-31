-- Enable pgvector for schema embeddings and migrate local fallback columns.

CREATE EXTENSION IF NOT EXISTS vector;

ALTER TABLE schema_embeddings
    ALTER COLUMN embedding TYPE VECTOR(1536)
    USING NULL::VECTOR(1536);

-- Vector index for schema similarity search.
CREATE INDEX IF NOT EXISTS idx_schema_embeddings_embedding
ON schema_embeddings USING ivfflat (embedding vector_cosine_ops);
