# lnshm

Have you ever use `/dev/shm`? It's pre-mounted ram drive in linux with capacity of `MAX_RAM / 2`.
In my machine it's fast as `1.2 GiB/s` per core. but almost-infinity endurance and delete when your pc restart.

In my machine I always use it to store build output e.g. `target` for rust or `dist` for typescript build but..  
I always have to run `mkdir /ramdisk/target && ln -sf /ramdisk/target ./target` everytime I clone or create a
project, so why not just automate it?

# Installation
> You need cargo (rust's package manager) to build
```
cargo install --git https://github.com/Wireless4024/lnshm
```

# Usage
```
USAGE:
    lnshm [OPTIONS] [LINK_TARGET]

ARGS:
    <LINK_TARGET>    target folder to link to ramdisk

OPTIONS:
    -c, --config <CONFIG>         Path to config file
        --generate <GENERATOR>    Generate completion script [possible values: bash, elvish, fish,
                                  powershell, zsh]
    -h, --help                    Print help information
    -i, --info                    Print information and exit
    -r, --remove                  Unlink / remove instead of create (ignore source option)
    -s, --source <SOURCE>         Path to source directory (copy content into ramdisk on mount)
        --system                  Run as system mode (eg. systemd hook on linux)
    -V, --version                 Print version information
```

# Features

+ [x] Config file
+ [x] Link directory
+ [x] Copy data from folder on create
    + [ ] Sync data from ramdisk back to source folder (cli)
+ [x] CLI
+ [ ] pre-build binary / integrate with system
+ [ ] Unit test?
