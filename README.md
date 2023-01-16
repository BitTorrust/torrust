# Torrust

## Description

Torrust is a simple BitTorrent client implemented in Rust as the final project of the networking course at Grenoble INP - ENSIMAG.

## Requirements

- libssl-dev: necessary for the `reqwest` crate

## Development dependencies

All development dependencies are expected to be installed and in your path.

- [aria2c](https://aria2.github.io/): aria2 is a command-line download utility that can
be used both as a peer or a seeder for .torrent files.
- [opentracker](https://github.com/wwwwg/opentracker): opentracker is a BitTorrent tracker.

## Building

To build the project, run:

```
cargo build --release
```

## Using Torrust

To execute Torrust, run:

```
cargo run --release -- your_torrent.torrent your_working_directory [--info|--debug]
```

If the working directory already contains the file to be downloaded, Torrust will seed the file.
If the working directory does not contain or partially contains the file to be downloaded, Torrust will attempt to
download the missing pieces while seeding the pieces it already has.

There are two log levels, info and debug. The default is no logs. If you want readable logs, run with --info. If you want specific logs, run with --debug.

If you need more details, a --help is available:

```
cargo run --release -- --help

A very humble Torrent client made with all our effort

Usage: torrust [OPTIONS] <TORRENT_FILE> <WORKING_DIRECTORY>

Arguments:
  <TORRENT_FILE>       The .torrent file path
  <WORKING_DIRECTORY>  The download path to store/upload the file described in .torrent

Options:
  -i, --info   Gives network peers information (bittorrent application, address IP, port, download/upload piece state)
  -d, --debug  Print minimal debug info
  -m, --mock   Communicate directly with three local peers using ports 2001, 2002 and 2003
  -h, --help   Print help information
```
## Performance Tests 

<!-- ¿Is necessary to test other file rather than 1Gbit.torrent? -->
**Test Torrust as leecher with one client**


1. Launch the script `one-seeder.sh`. 

The script will launch aria as seeder with the respective `torrent file` and the `working directory` where is stored the file to upload.
As an option, the listening port could be forced if desired.  
```
./scripts/one-seeder.sh <TORRENT_FILE> <WORKING_DIRECTORY> [LISTEN_PORT]
```

2. Launch torrust with:

```
cargo run --release -- <TORRENT_FILE> <WORKING_DIRECTORY> [--info|--debug]
```

**Test Torrust as seeder**

1. Launch the script `tracker.sh`

```
./scripts/tracker.sh
```

2. Launch torrust with

```
cargo run --release -- <TORRENT_FILE> <WORKING_DIRECTORY> [--info|--debug]
```

3. Launch Vuze and open the torrent file

4. At this moment Vuze will begin to download the file.


**Test Torrust as leecher - multi clients**

1. Go to the ./scripts/multi-client directory
2. launch ./partial.sh to configure the different file parts each aria will have

3. Launch the script named multi-seeders-iceberg.sh with the follow parameters

```
./scripts/multi-seeders-iceberg.sh <TORRENT_FILE> <WORKING_DIRECTORY> <WORKING_DIR_ARIAS>

Arguments:
  <TORRENT_FILE>       The .torrent file path
  <WORKING_DIRECTORY>  The download path to store/upload the file described in .torrent
  <WORKING_DIR_ARIAS>  The folder where are contained the aria subfolders, normally, it will be the /multi-client folder.
```




## Contact

[Jean Diego Silva Fontena](mailto:Jean-Diego.Silva-Fontena@grenoble-inp.org)

[Julien Liottard](mailto:Julien.Liottard@grenoble-inp.org)

[Juan Jose Duarte Garcia](mailto:Juan-Jose.Garcia-Duarte@grenoble-inp.org)
