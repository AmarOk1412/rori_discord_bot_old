**Disclaimer: This is a draft version**

# rori_discord_bot

![](https://travis-ci.org/AmarOk1412/rori_discord_bot.svg?branch=master)

RORI Discord Bot is an entry and an endpoint for _[RORI](https://github.com/AmarOk1412/rori/)_. This application is designed for a bot user on [Discord](http://discordapp.com/) based on this [library](https://github.com/SpaceManiac/discord-rs) to interact with _RORI_.

# Installation

This application requires _Rust_ (please read this [page](https://www.rust-lang.org/en-US/install.html)), _openssl_ and _libsodium_. To build the software, you just have to launch `cargo build` in this repository.

# Configuration

## Create a Discord Bot

Please read this [page](https://discordapp.com/developers/docs/topics/oauth2) to create a bot on Discord. When you have successfully created a bot on Discord, you can open _config_endpoint.json_ and write:

```json
"botname": "THE NAME OF YOUR BOT",
"channel": "THE CHANNEL ID ON DISCORD",
"token":"THE TOKEN OF YOUR BOT",
```

## Connect to rori_server

Note: you need to configure a `rori_server` first.

### Entry point side

_TODO: I need to remove this file_.<br>
You can configure the connection from _config_server.json_:

```json
{
 "ip":"IP of rori_server",
 "port":"port"
}
```

Moreover, you need to choose a secret (for the authentification) and a name for this entry point and write it in _config_endpoint.json_.

### rori_server side

You need to authorize the entrypoint to communicate with `rori_server`. In _config_server.json_ you need to add:

```json
"authorize": [
  {
    "name":"the name you choose",
    "secret":"sha256 of the secret you choose"
  }
]
```

## Authorize rori_server

You need to authorize `rori_server` to communicate with you. In _config_server.json_ you must add:

```json
"authorize": [
  {
    "name":"the name of the rori_server",
    "secret":"sha256 of the secret of the rori_server"
  }
]
```

## Tls configuration

All connections need to be secured. So you need to generate a private key and a certificate. On linux, you can run this following command: `openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem`. It will create a certificate (_cert.pem_) and a private key (_key.pem_). Now, you can add these files to _config_endpoint.json_.

## Final

The final _config_endpoint.json_ should look like this:

```json
{
 "ip":"0.0.0.0",
 "port":"1416",
 "rori_ip":"127.0.0.1",
 "rori_port":"1412",
 "owner":"*",
 "name":"rori_discord_bot",
 "compatible_types":"text",
 "cert":"key/cert.pem",
 "key":"key/key.pem",
 "secret": "secret",
 "botname": "THE NAME OF YOUR BOT",
 "channel": "THE CHANNEL ID ON DISCORD",
 "token":"THE TOKEN OF YOUR BOT",
 "authorize": [
   {
     "name":"rori_server",
     "secret":"2BB80D537B1DA3E38BD30361AA855686BDE0EACD7162FEF6A25FE97BF527A25B"
   }
 ]
}
```

# Execution

A binary is present in the _target/_ directory after a `cargo build` or you can execute `cargo run` in your shell.

# License

```
DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
        Version 2, December 2004

Copyright (C) 2016 Sébastien (AmarOk) Blin <https://enconn.fr>

Everyone is permitted to copy and distribute verbatim or modified
copies of this license document, and changing it is allowed as long
as the name is changed.

DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION

0\. You just DO WHAT THE FUCK YOU WANT TO.
```

# Contribute

Please, feel free to contribute to this project in submitting patches, corrections, opening issues, etc.
