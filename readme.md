# lnshm (WIP)

Have you ever use `/dev/shm`? It's pre-mounted ram drive in linux with capacity of `MAX_RAM / 2`.
In my machine it's fast as `1.2 GiB/s` per core. but almost-infinity endurance and delete when your pc restart.

In my machine I always use it to store build output e.g. `target` for rust or `dist` for typescript build but..  
I always have to run `mkdir /ramdisk/target && ln -sf /ramdisk/target ./target` everytime I clone or create a
project, so why not just automate it?

# Goal

> Note: still work in progress 😊

```shell
# mkdir /ramdisk/target && ln -sf /ramdisk/target ./target
# replace above command with
lnshm ./target
```

# Features

+ [x] Config file
+ [x] Link directory
+ [x] Copy data from folder on create
    + [ ] Sync data from ramdisk back to source folder (cli)
+ [ ] CLI
+ [ ] pre-build binary / integrate with system
+ [ ] Unit test?