# Reminisce [![Build Status](https://travis-ci.org/TomBebbington/reminisce.svg?branch=master)](https://travis-ci.org/TomBebbington/reminisce)

~~rusted joy, geddit?~~

Reminisce is a Rust library for detecting gamepads / joysticks and reading
input from them non-blockingly, without any external C dependencies.
It does this by using the native platform's raw Joystick API.

It aims to be cross-platform, but currently only supports Linux. However, its
API is simple and minimal so it is easy to implement.

[Documentation](http://tombebbington.github.io/reminisce/)
