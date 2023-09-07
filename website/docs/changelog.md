---
outline: 3
---
# `wallowa` changelog

Here are the [releases for `wallowa`](https://github.com/gunrein/wallowa/releases).

::: warning
The project will follow [semver 2.0](https://semver.org/) once `wallowa` reaches v1.0.

For now there will probably be breaking changes in minor versions but we'll do what we can to minimize and mitigate them.
:::

::: danger
The [DuckDB internal storage format](https://duckdb.org/internals/storage) is not yet stable.

Follow [these steps](https://duckdb.org/internals/storage#how-to-move-between-storage-formats)
before upgrading `wallowa` until DuckDB storage format stability is reached.
:::

### v0.1.0 on September 6, 2023 {#v0.1.0}

The initial version of `wallowa`. This version uses DuckDB v0.8.1.

The only prebuilt binary for this version is for MacOS on ARM.

[Download v0.1.0](https://github.com/gunrein/wallowa/releases/tag/v0.1.0)

#### Changes

- [CLI (command line interface)](cli) with `fetch`, `new`, `serve`, and `help` commands
- [Web UI](web-ui) with a dashboard, [GitHub Pull Request duration chart](sources/github), and index of sources
- Server to host the web UI
- [Documentation](http://www.wallowa.io/docs/)

#### Known issues

- Background data fetching in the server has a concurrency issue. During a fetch charts and other display elements that require data from the database will be slow to respond.
