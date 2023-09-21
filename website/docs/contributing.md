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

#### Update the changelog

The changelog records the changes made in each version. Create a new changelog entry in [website/docs/changelog.md](changelog) for each version.

- Use absolute URLs for links in the changelog so that the entry can be copy-paste to different locations without breaking the links
- The audience for the changelog are wallowa users so describe the changes in end-user terms rather than development terms

#### Create the release

To cut a new release from the `main` branch you'll create and push a new Git tag with the new version number. This will trigger the [Release](https://github.com/gunrein/wallowa/blob/main/.github/workflows/release.yaml) GitHub Action to build a Docker image for the new version and push it to [Docker Hub](https://hub.docker.com/).

1. Make sure `main` is up-to-date with all of the changes for the release including documentation updates and the [changelog entry](https://www.wallowa.io/docs/changelog.html)
2. Create a new "Release" on GitHub at https://github.com/gunrein/wallowa/releases/new
3. Click on "Choose a tag" and enter the new version number starting with a `v` (e.g. `v0.1.1`) and select "Create new tag: {version you entered} on publish"
4. Confirm that `main` is selected as the "Target"
5. Enter "wallowa {version you entered}" for the "Release title", e.g. "wallowa v0.1.1"
6. Copy-paste the changelog entry into the "Describe this release" field. Confirm that all links are absolute URLs.
7. Check the "Create a discussion for this release" checkbox so that a new discussion is created and choose the "Announcements" category
8. Click "Publish release" to publish the release and trigger the Docker image build GitHub Action

#### Publish the latest version of the documentation

1. `git branch website_production` - with the `main` branch up-to-date, switch to the `website_production`
2. `git pull origin main` - merge the changes from `main` into `website_production`
3. `git push origin website_production` - push the new content to GitHub. This will trigger a website rebuild and deploy on CloudFlare Pages.
4. Update the [Docker Hub repository overview] to add a link to the new version's Dockerfile in the "Supported tags and respective Dockerfile links" section
