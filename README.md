Device Blocker
==============

Limit screen time to children's various mobile devices by blocking internet
access on the family Wifi router.

This is the server which runs on the WiFi router. It has an API which lists
devices, opens and closes internet access for devices, and discovers new
devices on the router.

I run this on Lede 17.01.

It has a companion UI project:
[device-blocker-ui](https://github.com/darrint/device-blocker-ui)

Build/Installation
==================

Install Lede on a router and also download the correct Lede SDK for your
router.

Add the SDK to your path. For example:

`PATH=~/Downloads/lede-sdk-17.01.2-x86-64_gcc-5.4.0_musl-1.1.16.Linux-x86_64/staging_dir/toolchain-x86_64_gcc-5.4.0_musl-1.1.16/bin:$PATH`

Build the companion UI project:
[device-blocker-ui](https://github.com/darrint/device-blocker-ui)

```
# ... in the device-blocker-ui directory
npm install -g yarn
yarn
webpack --optimize-minimize --define process.env.NODE_ENV="'production'"
```

Next copy the resulting bundle.js and index.html files to this project's `src`
directory.

Use rustup to install a stable compiler for the architecture appropriate for
your router.

`cargo build --target x86_64-unknown-linux-musl --release`

Edit and copy some files from this project to your router:

* Edit and copy `known_devices.json` to `/etc/`
* Edit and copy `device_blocker.procd` to `/etc/init.d/device_blocker`
* Copy `target/x86_64-unknown-linux-musl/release/device-blocker` to `/root/`
* Make sure both `/root/device-blocker` and `/etc/init.d/device_blocker` are executable.

Use the LuCi interface to enable and start the service:

* Go to System -> Startup.
* Find device-blocker.
* Enable the service.
* Start the service.

ToDo
====
* Write files atomically.
* Figure out how to build this project with Xargo and deploy to mips and other router architecture.
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

