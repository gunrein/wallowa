# Get started

### Installation

`wallowa` deploys as a single statically linked binary with data stored in a
[DuckDB](http://duckdb.org/) database file.

Download the [latest release](https://github.com/gunrein/wallowa/releases), unzip it, and add it to your path.

### Start your first project

1. Navigate to the directory you'd like as the parent to your `wallowa` project directory
2. Run `wallowa new MY-PROJECT`, replacing `MY-PROJECT` with the name of your project
3. Change directory into the new project: `cd MY-PROJECT`
4. Add a [GitHub auth token to the `.env` file](configuration#github-auth-token)
5. Configure your project by editing `wallowa.config.toml` with a convenient text editor. The default file contains an overview of each setting and there is [documentation for all configuration options](configuration). 
6. Fetch data for the first time using [the CLI](cli): `wallowa fetch`
   ::: info This will take a while the first time if your repo(s) have a large number of PRs
7. Start the server: `wallowa serve`
8. Open your browser to http://localhost:9843/
9. Explore what's available and check out the documentation for the [web UI](web-ui) and [CLI](cli)

Thanks for using `wallowa`!
