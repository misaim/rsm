# rsm

Rust Session Manager is a CLI tool to copy an AWS ssm connection string to the clipboard for a given list of instances via the AWS SDK for Rust. 

Hacked together while understanding the Rust AWS SDK: improvement pr's and code reviews welcomed!

![rsm-example](https://github.com/misaim/rsm/assets/13842895/e48bb9c7-845f-4909-835d-fb456c721bf8)

## dependencies 
`cargo` - Note - The Rust AWS SDK is pretty dep-heavy right now. First builds will be slow, incremental builds should be significantly faster.

`cursive`, the TUI library used requires ncurses. Please install: https://github.com/gyscos/cursive/wiki/Install-ncurses

## build options

`make build`. Produces a dev build. Output binary is at `./target/debug/rsm`.

`make run` will build + run above binary. 

`make release` will do a clean release build, and produce an output binary at `.bin/rsm`. Sizes are around 9mb. 

`make clean` simply runs `cargo clean`.

## usage 
Requires a valid AWS credentials or config file.

`./rsm` should display all instances for current profile (default).
 - `-r region` sets region. Must be a valid AWS region.
 - `-p profile` sets profile. Note that a region must be specified as we don't handle defaulting. The AWS SDK is complicated!
 - `-v` sets verbose flag and is currently unused. 

Optional functions `write_json()` and `read_json()` are exported for your convenience but not called. Useful for troubleshooting. 

## theme

`cursive` theme file `style.toml` is loaded at compile time. 
