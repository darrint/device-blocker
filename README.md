Device Blocker
==============

Limit screen time to children's various mobile devices by blocking internet
access on the family Wifi router.

This is the server which runs on the WiFi router. It has an API which lists
devices, opens and closes internet access for devices, and discovers new
devices on the router.

It runs on OpenWRT.

It has a companion UI project:
[device-blocker-ui](https://github.com/darrint/device-blocker-ui)

Installation
============

Currently I've only ever run this on an `x86__64` device running a nightly build of
OpenWrt. It is written in Rust so in theory it should work well in a variety of
environments. For embedded work Rust only provides tier two support to `musl`. All
stable releases of OpenWRT are `uclibc`. I should be able to do better once OpenWRT
releases designated driver which is `musl` based.

Besides all that, the basic idea is to run this program on the router with appropriate
config files. A companion js web ui can use the api it exports to manage the devices.

ToDo
====
* Activate device internet access for a limited time. (i.e. Homework and chores are done, activate for 2hrs.)
* Write files atomically.
* Replace nickel with plain hyper.
* Figure out how to build this project Xargo and deploy to mips and other router architecture.
* Security? Require a login similar to OpenWRT's LuCi

License
=======

MIT License

Copyright (c) 2017 Darrin Thompson

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

