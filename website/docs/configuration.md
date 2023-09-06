---
outline: 3
---

# Configuration

Configuration is loaded from multiple places with precedence in this order:

1. CLI flags
2. The running shell's environment variables
3. Project-level environment configuration from the `.env` file in the project directory
4. Project-level configuration file (see [below for details](#config))

This project follows the [Command Line Interface Guidelines](https://clig.dev/).

### `config` {#config}

The configuration file to load. The configuration file can be expressed in one of multiple file formats: [TOML](https://toml.io/), [JSON](https://www.json.org/), [YAML](https://yaml.org/),
[INI](https://en.wikipedia.org/wiki/INI_file), [RON](https://github.com/ron-rs/ron),
or [JSON5](https://json5.org/).

- **Default**: `wallowa.config.[toml | json | yaml | ini | ron | json5]` in the working directory. The first configuration file found in the order listed here is used. The others are ignored.
- **CLI**: `wallowa --config CONFIG` where `CONFIG` is the path to the configuration file (`wallowa --config wallowa.config.toml` for example)
- **Environment variable**: `WALLOWA_CONFIG` (`WALLOWA_CONFIG=wallowa.config.toml` for example)

### `database` {#database}

The DuckDB database file to use. If the database file does not exist then it will be created. The special value `:memory:` can be used to create an in-memory database where no data is persisted to disk (all data is lost when the process exits). See the [DuckDB documentation on `connect`](https://duckdb.org/docs/connect.html) for more information.

- **Default**: `wallowa.db` in the working directory
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_DATABASE`

#### Example for the `wallowa.config.toml` file

```toml
database = "wallowa.db"
```

### `fetch.enabled`

Whether to fetch new data in the background. If this is disabled, then use the
`wallowa fetch` CLI command to fetch when the server is not running.

- **Default**: `true`
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_FETCH_ENABLED`

#### Example for the `wallowa.config.toml` file

```toml
[fetch]
enabled = false
```

### `fetch.interval`

The time interval to wait between fetching for additional data, in seconds.

- **Default**: `3600` (1 hour)
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_FETCH_INTERVAL`

#### Example for the `wallowa.config.toml` file

```toml
[fetch]
interval = 3600
```

### `github.auth.token` {#github-auth-token}

The auth token to use for authentication to the GitHub REST API. It is recommended to use a
[personal access token](https://docs.github.com/en/rest/overview/authenticating-to-the-rest-api?apiVersion=2022-11-28#authenticating-with-a-personal-access-token) with read-only access to each
of the [repos](#github-repos) being tracked.

- **Default**: none
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_GITHUB_AUTH_TOKEN`

#### Example as an environment variable or in the `.env` file

```sh
WALLOWA_GITHUB_AUTH_TOKEN='A TOKEN FROM GITHUB'
```

### `github.per_page` {#github-per-page}

The number of items to fetch per page of API results (maximum of 100).

- **Default**: `100`
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_GITHUB_PER_PAGE`

#### Example for the `wallowa.config.toml` file

This example sets the number of items to fetch per page to 50.

```toml
[github]
per_page = "50"
```

### `github.repos` {#github-repos}

The GitHub repositories to track.

- **Default**: `[]` (no repositories)
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_GITHUB_REPOS`

#### Example for the `wallowa.config.toml` file

This example tracks two repos: `open-telemetry/opentelemetry-rust` and `open-telemetry/opentelemetry-swift`.

```toml
[github]
repos = ["open-telemetry/opentelemetry-rust", "open-telemetry/opentelemetry-swift"]
```

### `log-format` {#log-format}

Set the log format.

- **Default**: `terminal`
- **CLI**: `wallowa --log-format=FORMAT` where `FORMAT` is one of the values listed above (`wallowa --log-format=full` for example)
- **Environment variable**: `WALLOWA_LOG_FORMAT` (`WALLOWA_LOG_FORMAT=json` for example)

Accepted values are:

- `terminal` - terminal-friendly human-readable basic log messages (the default)
- `full` - richer human-readable log messages
- `compact` - similar to `full`, but with less information
- `pretty` - multi-line version of `full`
- `json` - newline-delimited JSON logs

See https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/#formatters
for more details.

### Log level {#log-level}

Set the log level to output. See https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html for more on configuration options.

Note that setting the environment variable `RUST_BACKTRACE=1` can be used to include a backtrace in error output. See https://doc.rust-lang.org/std/backtrace/index.html for more details.

- **Default**: `INFO`
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_LOG` (`WALLOWA_LOG=debug` for example)

Accepted values are:

- `terminal` - terminal-friendly human-readable basic log messages (the default)
- `full` - richer human-readable log messages
- `compact` - similar to `full`, but with less information
- `pretty` - multi-line version of `full`
- `json` - newline-delimited JSON logs

See https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/#formatters
for more details.

### `server.host`

The network address to bind to.

- **Default**: "127.0.0.1"
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_SERVER_HOST`

#### Example for the `wallowa.config.toml` file

```toml
[server]
host = "127.0.0.1"
```

### `server.port`

The network port to bind to.

- **Default**: "9843"
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_SERVER_PORT`

#### Example for the `wallowa.config.toml` file

```toml
[server]
port = "9843"
```

### `server.response.compression.brotli`

Use brotli compression for HTTP server responses when requested by
the client.

- **Default**: false
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_SERVER_RESPONSE_COMPRESSION_BROTLI`

#### Example for the `wallowa.config.toml` file

```toml
[server.response.compression]
brotli = false
```

### `server.response.compression.deflate`

Use deflate compression for HTTP server responses when requested by
the client.

- **Default**: true
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_SERVER_RESPONSE_COMPRESSION_DEFLATE`

#### Example for the `wallowa.config.toml` file

```toml
[server.response.compression]
deflate = true
```

### `server.response.compression.gzip`

Use gzip compression for HTTP server responses when requested by
the client.

- **Default**: true
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_SERVER_RESPONSE_COMPRESSION_GZIP`

#### Example for the `wallowa.config.toml` file

```toml
[server.response.compression]
gzip = true
```

### `server.response.compression.level`

The compression level to use for HTTP server responses. Options are: `algo_default`, `best`, `fastest`.

`algo_default` uses the default compression level for the given compression algorithm. See https://docs.rs/tower-http/0.4.1/tower_http/enum.CompressionLevel.html#variant.Default for more.

- **Default**: "fastest"
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_SERVER_RESPONSE_COMPRESSION_LEVEL`

#### Example for the `wallowa.config.toml` file

```toml
[server.response.compression]
level = "fastest"
```

### `server.response.compression.zstd`

Use zstd compression for HTTP server responses when requested by
the client.

- **Default**: true
- **CLI**: this setting cannot be configured with a CLI argument
- **Environment variable**: `WALLOWA_SERVER_RESPONSE_COMPRESSION_ZSTD`

#### Example for the `wallowa.config.toml` file

```toml
[server.response.compression]
zstd = true
```
