# telewave

Browse and search live-audio recordings posted in Telegram channels.

Polls the channels you follow, logging in as your own account via
[TDLib](https://github.com/tdlib/td).

## Setup

TDLib is a git submodule and has to be built once before `cargo build` will link:

```sh
git submodule update --init
cmake -S vendor/td -B vendor/td/build -DCMAKE_BUILD_TYPE=Release \
      -DCMAKE_INSTALL_PREFIX:PATH="$PWD/tdlib-install"
cmake --build vendor/td/build --target install -j"$(nproc)"
```

Then copy `.env.example` to `.env` and fill in your `api_id` / `api_hash` from
[my.telegram.org](https://my.telegram.org) and the phone number to log in as.

```sh
cargo run
```
