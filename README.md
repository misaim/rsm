# rsm

A smallish binary to read EC2 instances and copy an AWS SSM connection string to the command line. 
Hacked together while understanding the Rust AWS SDK: don't shoot me.

## dependancies 
`cargo` - Note - The Rust AWS SDK is pretty heavy right now. First builds will be slow, incremental builds should be significantly faster.

`cursive`, the TUI library used requires ncurses. Please install: https://github.com/gyscos/cursive/wiki/Install-ncurses

## build options

`make build`. Output binary is at `./target/debug/rsm`.

`make run` will build + run. 

`make release` will do a clean build, and produce an output binary at `.bin/rsm`.

`make clean` simply runs `cargo clean`.

## usage 

`./rsm` should display all instances for current profile (default).
 - `-r region` sets region. Must be a valid AWS region.
 - `-p profile` sets profile. Note that a region must be specified as we don't handle defaulting. The AWS SDK is hard!
 - `-v` sets verbose flag and is currently unused. 
