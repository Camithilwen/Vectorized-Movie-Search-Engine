from colbert.infra import Run, RunConfig, ColBERTConfig
from colbert.modeling.checkpoint import Checkpoint
import json
import sys

def generate_embeddings(texts):
    try:
        with Run().context(RunConfig(nranks=1, experiment="temp")):
            config = ColBERTConfig(
                doc_maxlen=512,  # Reduced from 8192 for stability
                nway=2,
                similarity="cosine",
            )
            ckpt = Checkpoint(
                "colbert-ir/colbertv2.0",
                colbert_config=config
            )
            return ckpt.docFromText(texts, bsize=16)[0].tolist()  # Reduced batch size
    except Exception as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    try:
        input_texts = json.loads(sys.stdin.read())
        embeddings = generate_embeddings(input_texts)
        print(json.dumps(embeddings))
        sys.stdout.flush()  # Ensure output is flushed
    except Exception as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        sys.exit(1)
