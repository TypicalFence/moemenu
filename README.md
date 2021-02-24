# moemenu
Moemenu is a cute replacement for dmenu, written in rust. 

It aims to be simple, small, userfriendly and portable.

## Dependencies
- X11
- cairo
- a C compiler 

## Building

You can build it using cargo:
```
cargo build --release
```

## Config
You can change the looks of moemenu with a config file in the toml format.
Place the config file in your `XDG_CONFIG_HOME` (usually `$HOME/.config`).
There is an also a [example one](./etc/moemenu.toml).
