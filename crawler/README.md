# Telewave crawler

Rust application that authenticates with Telegram through TDLib and polls the
channels you follow.

Run the commands below from `crawler/`.

## Setup

TDLib is a git submodule and must be built once before the crawler can link.
Install Rust, CMake 3.24 or newer, a C++ compiler, OpenSSL, and zlib first.

```sh
git submodule update --init --recursive
cmake --fresh -S vendor/td -B vendor/td/build -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX:PATH="$PWD/tdlib-install"
cmake --build vendor/td/build --target install --parallel
```

`--fresh` refreshes any CMake cache containing paths from before the monorepo move.
An existing `tdlib-install/` can be reused without rebuilding TDLib.

Copy `.env.example` to `.env` and fill in your `api_id` / `api_hash` from
[my.telegram.org](https://my.telegram.org) and the phone number to log in as.

```sh
cp .env.example .env
cargo run
```

Credentials, TDLib session data in `.data/`, native libraries in `tdlib-install/`,
and Cargo build output in `target/` stay local to this directory and are ignored
by Git. To use another TDLib installation, set `TDLIB_DIR` to its absolute path.

From the repository root, use `npm run dev:crawler`, `npm run build:crawler`, or
`npm run check:crawler`. These commands run Cargo from this directory so relative
configuration and session paths resolve correctly.
