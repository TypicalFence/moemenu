# moemenu
Moemenu is a cute replacement for dmenu, written in rust. 

It aims to be simple, small, userfriendly and portable.

## Dependencies
- X11
- libxcb 
- cairo
- a C compiler 

## Building

You can build it using cargo:
```
cargo build --release
```

There are feature flags for removing bloat, check the Cargo.toml for details.

## Config
You can change the looks of moemenu with a config file in the toml format.
Place the config file in your `XDG_CONFIG_HOME` (usually `$HOME/.config`).
There is an also a [example one](./etc/moemenu.toml).

Should you want to disable the config feature you can easily edit the defaults in the [defaults.rs](./src/defaults.rs).
