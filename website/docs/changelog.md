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

### v0.3.0 on September 21, 2023 {#v0.3.0}

Wallowa v0.3.0 to use a Docker image for distribution. The image is available as [`gunrein/wallowa`](https://hub.docker.com/r/gunrein/wallowa) on Docker Hub.

To run this version use the command:

```sh
docker run -v .:/usr/wallowa:rw -p 127.0.0.1:9843:9843 --platform linux/amd64 gunrein/wallowa:0.3.0`
```

This version uses DuckDB v0.8.1 (the same version as Wallowa v0.2).

#### Changes

- Use a Docker image for distribution
- Use a GitHub Action to automatically build a new Docker image for the release when a new Git tag is pushed to GitHub

#### Known issues

- Background data fetching in the server has a concurrency issue. During a fetch charts and other display elements that require data from the database will be slow to respond.

### v0.2.0 on September 18, 2023 {#v0.2.0}

Wallowa v0.2.0 adds a chart for the [​count of closed GitHub Pull Requests by repo](https://www.unre.in/wallowa/docs/sources/github#closed-pr-count)​, improves messages when fetching, and fixes a bug.

Thanks to [@NoriSte](https://github.com/NoriSte) for contributing [PR #14](https://github.com/gunrein/wallowa/pull/14)!

This version uses DuckDB v0.8.1 (the same version as Wallowa v0.1).

The only prebuilt binary for this version is for MacOS on ARM.

[Download v0.2.0](https://github.com/gunrein/wallowa/releases/tag/v0.2.0)

#### Changes

- Fixed an incorrect https URL in the [get started](https://www.unre.in/wallowa/docs/get-started) content and CLI. Thanks to [@NoriSte](https://github.com/NoriSte) for [PR #14](https://github.com/gunrein/wallowa/pull/14)!
- Added a new chart for the [count of closed GitHub Pull Requests by repo](https://www.unre.in/wallowa/docs/sources/github#closed-pr-count)
- Added the count of closed GitHub Pull Requests by repo chart to the dashboard
- Improved messages for [CLI fetch](https://www.unre.in/wallowa/docs/cli#wallowa-fetch) when requests are made and when errors occur on the GitHub API fail
- Show error message when a fetch fails in the [sources web UI](https://www.unre.in/wallowa/docs/web-ui#sources)

#### Known issues

- Background data fetching in the server has a concurrency issue. During a fetch charts and other display elements that require data from the database will be slow to respond.

### v0.1.0 on September 6, 2023 {#v0.1.0}

The initial version of `wallowa`. This version uses DuckDB v0.8.1.

The only prebuilt binary for this version is for MacOS on ARM.

[Download v0.1.0](https://github.com/gunrein/wallowa/releases/tag/v0.1.0)

#### Changes

- [CLI (command line interface)](https://www.unre.in/wallowa/docs/cli) with `fetch`, `new`, `serve`, and `help` commands
- [Web UI](https://www.unre.in/wallowa/docs/web-ui) with a dashboard, [GitHub Pull Request duration chart](https://www.unre.in/wallowa/docs/sources/github), and index of sources
- Server to host the web UI
- [Documentation](https://www.unre.in/wallowa/docs/)

#### Known issues

- Background data fetching in the server has a concurrency issue. During a fetch charts and other display elements that require data from the database will be slow to respond.
