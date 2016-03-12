# Reminisce
[![Build Status](https://travis-ci.org/TomBebbington/reminisce.svg?branch=master)](https://travis-ci.org/TomBebbington/reminisce)
[![Cargo Version](http://meritbadge.herokuapp.com/reminisce)](https://crates.io/crates/reminisce)
[![Gitter Chat](https://badges.gitter.im/TomBebbington/reminisce.png)](https://gitter.im/TomBebbington/reminisce)

~~rusted joy, geddit?~~

Reminisce is a primarily event-based Rust library for detecting gamepads / joysticks
and reading input from them non-blockingly, without any external C dependencies.
It does this by using the native platform's raw Joystick API.

This is intended for use with Glutin, since it doesn't implement a Joystick API.

## Documentation
Documentation is available [here](https://tombebbington.github.io/TomBebbington).

## Supported platforms
+ Linux (using the Joystick API or using SDL)
+ Windows Vista or higher (XInput should work but for the moment just SDL is supported)
