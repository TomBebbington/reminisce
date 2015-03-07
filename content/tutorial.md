+++
date = "2015-03-07T19:47:18Z"
draft = true
title = "Tutorial"

+++

## Setting up

To get started using Reminisce, add the following to your Cargo.toml:

``` toml
[dependencies.reminisce]
git = "https://github.com/TomBebbington/reminisce"
```
If you are using SDL, you should enable the `sdl` feature by adding the following:

``` toml
features = [ "sdl" ]
```
This will make reminisce wrap SDL's joystick API instead of needlessly
re-inventing the wheel.

## Coding

If it does not exist already, create a `main.rs` and add the following to it:

``` rust
extern crate reminisce;
use reminisce::*;
```

This pulls in the Reminisce library and imports its contents so you can use the
names without using the full paths.
Then, declare the main function:

``` rust
fn main() {

}
```

Inside it put the following:

``` rust
let mut joysticks = scan();
```

This scans for joysticks and returns them as a vector, then stores them in
the mutable variable `joysticks`. This has to be mutable for us to check its
events later.

``` rust
loop {
    for js in &mut joysticks {
        for event in js.iter() {
            println!("{:?} fired", event)
        }
    }
}
```
This continuously loops through the joysticks vector and for each one loops through its
pending events and prints them all.

Now, time to build!

## Building

Now open a Terminal or Command Prompt, plug in your joystick or gamepad, then type in:

``` bash
cargo run
```

Now press some buttons on your gamepad and move some sticks - you should see
the corresponding events in the output of the command you just entered.
