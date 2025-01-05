# dagster-contrib-chroma

A dagster module that provides integration with [chroma](https://www.trychroma.com/).

This module provides the `ChromaLocalResource` (for a self-hosted instance), 
and the `ChromaHttpResource` (to connect to a Chroma server via HTTP).

## Installation

The `dagster_contrib_chroma` module is available as a PyPI package - install with your preferred python 
environment manager (We recommend [uv](https://github.com/astral-sh/uv)).

```
source .venv/bin/activate
uv pip install dagster_contrib_chroma
```

## Example Usage (Local Chroma Instance)

(Based on the [Chroma getting started guide](https://docs.trychroma.com/docs/overview/getting-started))

```python
from dagster import Definitions, asset
from dagster_contrib_chroma import ChromaResource, LocalConfig

@asset
def my_table(chroma: ChromaResource):
    with chroma.get_client() as chroma_client:
        collection = chroma_client.create_collection("fruits")

        collection.add(
            documents=[
                "This is a document about oranges", 
                "This is a document about pineapples",
                "This is a document about strawberries",
                "This is a document about cucumbers"],
            ids=["oranges", "pineapples", "strawberries", "cucumbers"],
        )

        results = collection.query(
            query_texts=["hawaii"],
            n_results=1,
        )

defs = Definitions(
    assets=[my_table],
    resources={
        "chroma": ChromaResource(
            connection_config=LocalConfig(persistence_path="./chroma")
        )
    }
)
```

## Example Usage (Connecting via HTTP)

(Based on the [Chroma getting started guide](https://docs.trychroma.com/docs/overview/getting-started))

```python
from dagster import Definitions, asset
from dagster_contrib_chroma import ChromaResource, HttpConfig

@asset
def my_table(chroma: ChromaResource):
    with chroma.get_client() as chroma_client:
        collection = chroma_client.create_collection("fruits")

        collection.add(
            documents=[
                "This is a document about oranges", 
                "This is a document about pineapples",
                "This is a document about strawberries",
                "This is a document about cucumbers"],
            ids=["oranges", "pineapples", "strawberries", "cucumbers"],
        )

        results = collection.query(
            query_texts=["hawaii"],
            n_results=1,
        )

        return MaterializeResult(
            metadata={
                "result": '.'.join(results["ids"][0])
            }
        )

defs = Definitions(
    assets=[my_table],
    resources={
        "chroma": ChromaResource(
            connection_config=HttpConfig(host="localhost", port=8000)
        )
    }
)
```

## Development

The `Makefile` provides the tools required to test and lint your local installation.

```sh
make test
make ruff
make check
```