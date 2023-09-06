---
outline: deep
---
# Data analysis

### Analyze with DuckDB {#duckdb}

Thanks to [DuckDB](https://duckdb.org/) the data collected by `wallowa` can be analyzed in several ways including:

- Query using
  [SQL](https://duckdb.org/docs/guides/index#duckdb-sql),
  the [ASOF join](https://duckdb.org/docs/guides/sql_features/asof_join) (see [GitHub Pull Request duration for an example](sources/github#pull-duration)),
  [full text search](https://duckdb.org/docs/guides/sql_features/full_text_search),
  [Ibis](https://duckdb.org/docs/guides/python/ibis),
  [Polars](https://duckdb.org/docs/guides/python/polars),
  [Vaex](https://duckdb.org/docs/guides/python/vaex),
  and [DataFusion](https://duckdb.org/docs/guides/python/datafusion)
- Explore using
  [Jupyter Notebooks](https://duckdb.org/docs/guides/python/jupyter),
  [DBeaver](https://duckdb.org/docs/guides/sql_editors/dbeaver),
  [Tableau](https://duckdb.org/docs/guides/data_viewers/tableau),
  and [YouPlot](https://duckdb.org/docs/guides/data_viewers/youplot)
- Export to
  [Parquet](https://duckdb.org/docs/guides/import/parquet_export) and [Parquet on S3](https://duckdb.org/docs/guides/import/s3_export),
  [CSV](https://duckdb.org/docs/guides/import/csv_export),
  [JSON](https://duckdb.org/docs/guides/import/json_export),
  [Excel](https://duckdb.org/docs/guides/import/excel_export),
  [Pandas](https://duckdb.org/docs/guides/python/export_pandas),
  [Apache Arrow](https://duckdb.org/docs/guides/python/export_arrow)

Follow the [DuckDB guides](https://duckdb.org/docs/guides/index) to learn more.

### Tables

There is only one table in `wallowa` so far.

#### `wallowa_raw_data` {#wallowa_raw_data}

This table stores the raw JSON payloads from the APIs that data is fetched from. Queries can use
the [DuckDB JSON extension](https://duckdb.org/docs/extensions/json) to extract the data of interest from the payloads. See:

- [Shredding Deeply Nested JSON, One Vector at a Time](https://duckdb.org/2023/03/03/json.html) for a demo and tutorial of the DuckDB functionality
- [GitHub Pull Request duration](sources/github#pull-duration) for an example using data from the  GitHub Pulls API

```sql
CREATE SEQUENCE seq_wallowa_raw_data;
CREATE TABLE IF NOT EXISTS wallowa_raw_data (
    id INTEGER PRIMARY KEY DEFAULT NEXTVAL('seq_wallowa_raw_data'),
    created_at TIMESTAMP DEFAULT now() NOT NULL,
    loaded_at TIMESTAMP,
    "data_source" VARCHAR,
    data_type VARCHAR,
    metadata JSON,
    "data" VARCHAR
)
```
