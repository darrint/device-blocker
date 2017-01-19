Device Blocker
==============

Device Blocker blocks individual devices while running on a router.  It
identifies the devices based on their mac addresses.  It works by updating an
iptables chain to decide what to do with each device, and unknown devices.

It also scans a dhcp leases file to ease adding new devices by mac address.

Installation
============

Currently I've only ever run this on an `x86__64` device running a nightly build of
OpenWrt. It is written in Rust so in theory it should work well in a variety of
environments. For embedded work Rust only provides tier two support to `musl`. All
stable releases of OpenWRT are `uclibc`. I should be able to do better once OpenWRT
releases designated driver which is `musl` based.

Besides all that, the basic idea is to run this program on the router with appropriate
config files. A companion js web ui can use the api it exports to manage the devices.

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

