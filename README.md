# Reminisce
[![Build Status](https://travis-ci.org/TomBebbington/reminisce.svg?branch=master)](https://travis-ci.org/TomBebbington/reminisce)
[![Cargo Version](http://meritbadge.herokuapp.com/reminisce)](https://crates.io/crates/reminisce)
[![Gitter Chat](https://badges.gitter.im/TomBebbington/reminisce.png)](https://gitter.im/TomBebbington/reminisce)

~~rusted joy, geddit?~~

Reminisce is a Rust library for detecting gamepads / joysticks and reading
input from them non-blockingly, without any external C dependencies.
It does this by using the native platform's raw Joystick API.

[Documentation](http://www.rustdox.com/TomBebbington/reminisce/reminisce/)

## Supported platforms
+ Linux (using the Joystick API or using SDL)
+ Windows Vista or higher (using XInput, untested or using SDL)
+ Emscripten (using the Gamepad API, will work as soon as Rust supports it)
