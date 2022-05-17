# Privacy Chess ♔

Ever get tired of chess.com sharing your catastrophic blunders with the world? Introducing privacy-chess, an end-to-end encrypted chess game
using the [tari](https://github.com/tari-project/tari) testnet network.

## Build

`cargo build --release`

Dependencies: 

```
libasound-dev 
libxcb-shape0-dev
libxcb-xfixes0-dev
```

## Usage

Install the tor proxy. On mac use `brew install tor`.
The tor proxy is automatically started.

```shell
# Simply run p2p-chess, `.p2pchess` will be created in that folder with your secret network identity
# and saved games.
p2p-chess

# If you prefer to use an existing tor proxy, use:
p2p-chess --local-tor-control-port <tor_control_port> # or -t 9051 for short
```

## Status

This is very alpha toy software and has been put together as quickly as possible. There are bugs and there are
no guarantees that you'll be able to finish each game. Bug reports and PRs are appreciated!
