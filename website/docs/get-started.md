# Get started

### Installation

`wallowa` deploys as a [Docker](https://www.docker.com/) container with data stored in a
[DuckDB](http://duckdb.org/) database file. If you'd rather not use Docker you can [build a `wallowa` binary from source](#build-from-source).

### Start your first project

1. Navigate to the directory you'd like as the parent to your `wallowa` project directory
2. Run `docker run -v .:/usr/wallowa:rw -p 127.0.0.1:9843:9843 gunrein/wallowa new MY-PROJECT`, replacing `MY-PROJECT` with the name of your project
3. Change directory into the new project: `cd MY-PROJECT`
4. Add a [GitHub auth token to the `.env` file](configuration#github-auth-token)
5. Configure your project by editing `wallowa.config.toml` with a convenient text editor. The default file contains an overview of each setting and there is [documentation for all configuration options](configuration). 
6. Fetch data for the first time using [the CLI](cli): `docker run -v .:/usr/wallowa:rw -p 127.0.0.1:9843:9843 gunrein/wallowa fetch`
   ::: info This will take a while the first time if your repo(s) have a large number of PRs
7. Start the server: `docker run -v .:/usr/wallowa:rw -p 127.0.0.1:9843:9843 gunrein/wallowa gunrein/wallowa`
8. Open your browser to http://localhost:9843/
9. Explore what's available and check out the documentation for the [web UI](web-ui) and [CLI](cli)

Thanks for using `wallowa`!

### Build from source {#build-from-source}

1. Install build-time dependencies
   - A recent version of [NPM](https://nodejs.org/en/download)
   - A recent version of the [Rust toolchain](https://www.rust-lang.org/learn/get-started)
1. Download the [source code for the tagged version you're building](https://github.com/gunrein/wallowa/tags)
1. In the root directory of the source code, run:
   1. `npm install`
   1. `npm run build`
1. Use the newly-built binary at `target/release/wallowa`
