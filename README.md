# Privacy Chess ♔

Ever get tired of chess.com knowing about your every blunder? Introducing privacy-chess, an end-to-end encrypted chess game
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

You'll need to run the tor proxy. On mac use `brew install tor` and run

```shell
tor  --controlport 127.0.0.1:9051 --SocksPort 9050 
```

then run

```shell
# Simply run p2p-chess, `.p2p-chess` will be created in that folder with your secret network identity
# and saved games.
p2p-chess
```

## Status

This is very alpha toy software and has been put together as quickly as possible. There are bugs and there are
no guarantees that you'll be able to finish each game. Bug reports are appreciated!
