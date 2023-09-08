---
outline: 3
---

# Introduction

`wallowa` is (the start of) a tool that I've wanted in my past two engineering leadership roles. It is focused on providing straightforward and transparent insights into system-level behaviors of your [Software Development Life Cycle (SDLC)](https://en.wikipedia.org/wiki/Software_development_process) with both [out-of-the-box DORA/SPACE-like measures](web-ui) and [flexible ad hoc queries using SQL or various dataframe dialects](data-analysis). It is designed to give anyone on your team flexible access to query your software operations and development data using famliar tools.

Transparency, openness, and empowerment are all intentionally designed into `wallowa`.

[Get started](get-started) or learn more about the [philosophy behind the tool](#philosophy).

### Features

Key system-level DORA/SPACE-like measurements are available [out of the box](sources/) to provide immediate insight into the bottlenecks in
your tooling and processes. Go further by using [SQL, data frames, and other familiar data analysis tools](data-analysis) to query any measured aspect of your SDLC that you can think of. Specific features include:

- Chart the [30 day rolling average time to merge GitHub Pull Requests](sources/github#pull-duration) for a given time range
- Automatically [fetch GitHub Pull Request (PR)](cli#wallowa-fetch) from the [repos you configure](configuration#github-repos)

### `wallowa` is **not** a "productivity" measurement tool {#philosophy}

`wallowa` is expressly not intended for "productivity" measurement. Instead, it is a tool to help you and everyone on your team understand your SDLC quantitatively. These measurements can help you indentify opportunities for continued improvement, often the bottlenecks in your system.

Measuring the "productivity" of individuals in software development is misguided and fundamentally flawed. Instead, measuring the behavior of the system(s) that support your people throughout the SDLC will highlight opportunities for improvement that can lead to higher satisfaction for developers and users, improved quality, and more impactful output. Making those improvements takes investment in the areas identified. Using data that shows the bottleneck can help make a case for the needed investment. Improvement can be quantified as it happens since you have a baseline.

The performance and contribution of each individual is important, of course, but measuring "productivity" doesn't make sense. System-level behaviors tend to dominate a group's achievement (especially as the group gets larger) and performance+contribution are complex emergent properties that involve many factors. Individual performance and contribution are best addressed through individual development of competence and motivation in the context of the team(s) and project(s) that people are involved in, often with the guidance and support of a manager, mentor, or peer. Here's a great example from Dan North: [The Worst Programmer I Know](https://dannorth.net/2023/09/02/the-worst-programmer/) and a great pair of articles from Kent Beck and Gergely Orosz (Kent's [part 1](https://tidyfirst.substack.com/p/measuring-developer-productivity), [part 2](https://tidyfirst.substack.com/p/measuring-developer-productivity-440) & Gergely's [part 1](https://newsletter.pragmaticengineer.com/p/measuring-developer-productivity), [part 2](https://newsletter.pragmaticengineer.com/p/measuring-developer-productivity-part-2)) in response to [McKinsey publishing their methodology for measuring developer "productivity"](https://www.mckinsey.com/industries/technology-media-and-telecommunications/our-insights/yes-you-can-measure-software-developer-productivity).

Resources that inspired these views:

- "Accelerate" by Forsgren, Humble, Kim
- "Out of the Crisis" by Deming
- "The Goal" by Goldratt and "The Phoenix Project" by Kim, Behr, Stafford

### Advice on careful, intentional use of measurement

Like any tool, measurement can cause more harm than benefit if used inappropriately. Here is some advice on using measurement effectively for your SDLC.

- Approach measurement in the context of feedback loops (see [OODA loop](https://en.wikipedia.org/wiki/OODA_loop) for a useful example). Does the measure provide quick and actionable insight into an area to improve? Try it. If not, don't bother.
- Measure your feedback loop(s) in order to bring attention to shortening them. Two of the [DORA metrics](https://cloud.google.com/blog/products/devops-sre/using-the-four-keys-to-measure-your-devops-performance), Deployment Frequency and Lead Time for Changes, are practical and empirically-justified measures to use.
- Be data-enabled, not data-driven. It takes knowledge, data, and intuition to achieve great outcomes.
- Align your measurements and focus areas to your overall organizational/business goals so there is a clear line from the overall goals to the small number of measurements you're optimizing for.
- Talk with developers, designers, and PMs about where they experience friction, in general or in the context of specific metrics. They know best what’s holding them back. In larger organizations, fan out these chats with your managers. Surveys can help at high organizational scale or in low trust environments (if anonymous), but nothing replaces the insights that come from high trust dialogue.
- Synthesize and share your thinking and any results openly and transparently. Include details on which insights you’re taking action on, if any, why you picked those areas to focus, and how you're taking action.
- Only use metrics for short-term goals or standards. This avoids stagnant thinking, entrenching a status quo, or the proliferation of gamed incentives that tends to happen with long-term metrics.
- Metrics can and will be gamed; it is human nature (see [Goodhart's law](https://en.wikipedia.org/wiki/Goodhart%27s_law)). Pick metrics that, when gamed, will lead to positive behaviors anyway. For example, short Pull/Merge Request merge times drives the positive behavior of keeping PRs smaller and feedback loops shorter. When people find ways to game that metric, it'll probably still be a net positive outcome.
- Use counterbalancing metrics to mitigate some of the problems with gaming a specific metric. Continuing the example of short Pull/Merge Request merge times, optimizing for that metric alone can lead to cursory reviews or many interruptions. Counterbalance that behavior by finding a way to measure the number of interuptions or quality of reviews.
- Review all of the measures you've been keeping an eye on every month or two, depending on your organization’s cadence, to see where you’d like to focus next, if anywhere.
- Keep only a small number of metrics in focus or with set goals at any given time. Fewer metrics is better than many. Cull metrics that are not useful at the moment.

Please open a [documentation Pull Request](contributing#documentation) to add advice or refine/debate these points.

### Alternative tools

There are other tools available for similar insights (in lexicographic order). You can also build something similar yourself.

- [Apache DevLake](https://devlake.apache.org/)
- [Code Climate Velocity](https://codeclimate.com/velocity)
- [Cortex](https://www.cortex.io/)
- [DevStats](https://www.devstats.com/)
- [DX](https://getdx.com/)
- [Faros](https://www.faros.ai/)
- [Four Keys](https://github.com/dora-team/fourkeys)
- [Harness](https://www.harness.io/)
- [Haystack](https://www.usehaystack.io/)
- [Jellyfish](https://jellyfish.co/)
- [LinearB](https://linearb.io/)
- [PinPoint](https://pinpoint.com/)
- [Pluralsight Flow](https://www.pluralsight.com/product/flow)
- [Propelo](https://www.propelo.ai/)
- [Screenful](https://screenful.com/) 
- [Sleuth](https://www.sleuth.io/)
- [Swarmia](https://www.swarmia.com/)
- [Uplevel](https://uplevelteam.com/)
- [WayDev](https://waydev.co/)

Please open a [documentation Pull Request](contributing#documentation) to add other tools to this list.

### Thank you to these projects {#thank-you}

This project would take a much larger amount of effort (that I probably wouldn't undertake) without these great open projects to build upon. Thank you!

- [AlpineJS](https://alpinejs.dev/)
- [anyhow](https://docs.rs/anyhow/latest/anyhow/)
- [Apache Arrow](https://arrow.apache.org/)
- [axum](https://docs.rs/axum/latest/axum/) and [axum-extra](https://docs.rs/axum-extra/latest/axum_extra/)
- [chrono](https://docs.rs/chrono/latest/chrono/)
- [clap](https://docs.rs/clap/latest/clap/)
- [config](https://docs.rs/config/latest/config/)
- [CSS](https://www.w3.org/Style/CSS/specs.en.html)
- [daisyUI](https://daisyui.com/)
- [Diátaxis](https://diataxis.fr/)
- [dotenvy](https://docs.rs/dotenvy/latest/dotenvy/)
- [DuckDB](https://duckdb.org/) and [duckdb-rs](https://docs.rs/duckdb/latest/duckdb/)
- [esbuild](https://esbuild.github.io/)
- [futures](https://docs.rs/futures/latest/futures/)
- [HTML](https://html.spec.whatwg.org/)
- [HTMX](https://htmx.org/)
- [inquire](https://docs.rs/inquire/latest/inquire/)
- [Javascript](https://tc39.es/ecma262/) and [Typescript](https://www.typescriptlang.org/)
- [mime_guess](https://docs.rs/mime_guess/latest/mime_guess/)
- [minijinja](https://docs.rs/minijinja/latest/minijinja/) and [minijinja-autoreload](https://docs.rs/minijinja-autoreload/latest/minijinja_autoreload/)
- [Observable Plot](https://observablehq.com/plot/)
- [parse_link_header](https://docs.rs/parse_link_header/latest/parse_link_header/)
- [r2d2](https://docs.rs/r2d2/latest/r2d2/)
- [reqwest](https://docs.rs/reqwest/latest/reqwest/)
- [Rust](https://www.rust-lang.org/) and friends like [Cargo](https://doc.rust-lang.org/cargo/), [rustup](https://rustup.rs/), etc.
- [rust-embed](https://docs.rs/rust-embed/latest/rust_embed/)
- [serde](https://docs.rs/serde/latest/serde/), [serde_json](https://docs.rs/serde_json/latest/serde_json/), [serde_yaml](https://docs.rs/serde_yaml/latest/serde_yaml/)
- [TailwindCSS](https://tailwindcss.com/)
- [tokio](https://docs.rs/tokio/latest/tokio/)
- [tower-http](https://docs.rs/tower-http/latest/tower_http/)
- [tracing](https://docs.rs/tracing/latest/tracing/)
- [tracing-subscriber](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/)
- [VitePress](https://vitepress.dev/)

and all of the projects these tools depend on.
