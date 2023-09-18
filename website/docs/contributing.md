---
outline: 3
---

# Contributing

Contributions are welcome. For significant contributions, please open a [GitHub issue](https://github.com/gunrein/wallowa/issues) with your idea before putting significant work into it. That will give us a chance to discuss the idea before you get deeply into it.

### Documentation or website {#documentation}

The [Diátaxis](https://diataxis.fr/) approach to authoring documentation is used for content.
See also the [HackerNews discussion of The Surprising Power of Documentation](https://news.ycombinator.com/item?id=36287809) for other tips.

[VitePress](https://vitepress.dev/) is the documentation tool used.

#### Propose changes with a GitHub Pull Request

Follow the [GitHub Pull Request approach](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests)
to propose improvements to the documentation.

Please prefix the name of the branch with `docs`.

#### Development environment

Setup your development environment by running `npm install` in the root directory of the repo.

Run the documentation development server with `npm run docs:dev`.

#### Deploy the latest documentation & website content

[Cloudflare Pages](https://pages.cloudflare.com/)
are used to host the website and documentation. Deployment is automatic when any changes are merged into the `website_production` branch. There is no need to build the website and documentation locally/elsewhere, it will be generated automatically from the latest source code by the deploy process.

### Improvements to the tool

To add a new source, check out how the GitHub source works in:

- `src/github/*`
- `src-web/pr_duration.ts`
- `templates/github/*`
- `static/github/*`

To make other server-side improvements, explore `src/*`.

To make other browser-side improvements, explore `src-web/*`.

Please include documentation updates related to in the same Pull Request.

The overview of the [architecture](architecture) may be helpful.

#### Propose changes with a GitHub Pull Request

Follow the [GitHub Pull Request approach](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests)
to propose improvements to the tool.

#### Development environment

To setup your development environment:

- Install [rustup](https://rustup.rs/) (for Rust and Cargo)
- Install NPM
- Run `npm install` in the root directory of the repo

Commands to use during development:

- `npm run dev` - runs the web server and watches for changes in `src-web` and `templates` (it doesn't watch for Rust changes, so you'll need to restart the server manually to see changes in Rust)
- `npm build` - build a release binary
- `cargo test`
- `npm run fetch` - run the CLI `fetch` command

### Ship a new release

TODO
