# konachan-wallpaper
[![GitHub actions](https://github.com/LuckyTurtleDev/konachan-wallpaper/workflows/Rust/badge.svg)](https://github.com/LuckyTurtleDev/konachan-wallpaper/actions?query=workflow%3ARust)
[![crates.io](https://img.shields.io/crates/v/konachan-wallpaper.svg)](https://crates.io/crates/konachan-wallpaper)
[![License Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![konachan-wallpaper on deps.rs](https://deps.rs/repo/github/LuckyTurtleDev/konachan-wallpaper/status.svg)](https://deps.rs/repo/github/LuckyTurtleDev/konachan-wallpaper)

Program to download and set desktop wallpaper from [Konachan.net](https://konachan.net/).

The program download wallpapers to your local storage and can set them as wallpaper.
This allow you to set new wallpaper even if your pc is temporary offline.

## Commands:
* `konachan-wallpaper download`: download 200 wallpapers from konachan.net and save their path to a list (will override a existing list)
* `konachan-wallpaper set`: set background to a random wallpaper from the list

## Configuration:
You need to create a `config.txt` file.
This file should include tags seperated by space.
Tags with does start with `-` will, be blacklisted.
Wildcard are not supported.
The file can be empty, at this case every image will be used.
You can add as many tags, as you want. The limtation of the website does not matter.

Example `config.txt`:
```txt
hatsune_miku headphones -underwear
```

The program support only [tags](https://konachan.net/tag?name=&type=&order=count) everthing else like `rating:s` or `order:score` is not supported and leads to untested behaviour.
Current the rating save is hardcoded.

## Limitations: 
The programm is current in a very early stage and many thing are not suppored or are hardcoded at the moment.
Also the format of the config file will probably change multiple times.
Current the rating `save`, the picture count `200` and the order `latest` is hardcoded.

## Installation:
Current are no prebuild binaries available. You must build konachan-wallpaper by yourself. See below.

### Building:

* Install [rust](https://www.rust-lang.org/tools/install).

* To build and install mstickereditor execute the following command:
```bash
cargo install --locked konachan-wallpaper
```
Check out [rust doc](https://doc.rust-lang.org/cargo/commands/cargo-install.html) for more information about `cargo install`.
* You can uninstall rust now.
