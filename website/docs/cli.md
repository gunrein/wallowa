---
outline: 3
---
# Command line interface

For an overview of CLI commands, run `wallowa help`.

For help with a specific command, run `wallowa help <COMMAND>` where `<COMMAND>` is the name of the
command to view the help for.

### `wallowa help`

```sh
A tool for measuring aspects of your Software Development Life Cycle (SDLC).

Usage: wallowa [OPTIONS] [COMMAND]

Commands:
  fetch  Fetch the latest data from configured sources
  new    Create a new project in an new directory
  serve  Serve the web app
  help   Print this message or the help of the given subcommand(s)

Options:
      --config <CONFIG>          Set a custom config file [env: WALLOWA_CONFIG=] [default: wallowa.config.toml]
      --log-format <LOG_FORMAT>  Set the log format. Accepted values are:
                                 - `terminal` - terminal-friendly human-readable basic log messages (the default)
                                 - `full` - richer human-readable log messages
                                 - `compact` - similar to `full`, but with less information
                                 - `pretty` - multi-line version of `full`
                                 - `json` - newline-delimited JSON logs
                                 See https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/#formatters
                                 for more details. [env: WALLOWA_LOG_FORMAT=] [default: terminal]
  -h, --help                     Print help
  -V, --version                  Print version
```

### `wallowa fetch`

```sh
Fetch the latest data from configured sources

Usage: wallowa fetch [OPTIONS]

Options:
      --config <CONFIG>          Set a custom config file [env: WALLOWA_CONFIG=] [default: wallowa.config.toml]
      --log-format <LOG_FORMAT>  Set the log format. Accepted values are:
                                 - `terminal` - terminal-friendly human-readable basic log messages (the default)
                                 - `full` - richer human-readable log messages
                                 - `compact` - similar to `full`, but with less information
                                 - `pretty` - multi-line version of `full`
                                 - `json` - newline-delimited JSON logs
                                 See https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/#formatters
                                 for more details. [env: WALLOWA_LOG_FORMAT=] [default: terminal]
  -h, --help                     Print help
```

### `wallowa new`

```sh
Create a new project in an new directory

Usage: wallowa new [OPTIONS] <PATH>

Arguments:
  <PATH>  The path of the new project directory

Options:
      --config <CONFIG>          Set a custom config file [env: WALLOWA_CONFIG=] [default: wallowa.config.toml]
      --log-format <LOG_FORMAT>  Set the log format. Accepted values are:
                                 - `terminal` - terminal-friendly human-readable basic log messages (the default)
                                 - `full` - richer human-readable log messages
                                 - `compact` - similar to `full`, but with less information
                                 - `pretty` - multi-line version of `full`
                                 - `json` - newline-delimited JSON logs
                                 See https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/#formatters
                                 for more details. [env: WALLOWA_LOG_FORMAT=] [default: terminal]
  -h, --help                     Print help
```

### `wallowa serve`

:::danger
The server should not be exposed directly to the Internet since it has not been hardened for that environment. Run a proxy in front of the server if you choose to expose it to the Internet.
:::

```sh
Serve the web app

Usage: wallowa serve [OPTIONS]

Options:
      --config <CONFIG>          Set a custom config file [env: WALLOWA_CONFIG=] [default: wallowa.config.toml]
      --log-format <LOG_FORMAT>  Set the log format. Accepted values are:
                                 - `terminal` - terminal-friendly human-readable basic log messages (the default)
                                 - `full` - richer human-readable log messages
                                 - `compact` - similar to `full`, but with less information
                                 - `pretty` - multi-line version of `full`
                                 - `json` - newline-delimited JSON logs
                                 See https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/#formatters
                                 for more details. [env: WALLOWA_LOG_FORMAT=] [default: terminal]
  -h, --help                     Print help
```
