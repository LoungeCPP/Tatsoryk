# Tatsoryk
A Lounge&lt;Discord&gt; gaem project that gets completed.

The loungescussion and circlejerk about The Game is [here](https://forum.loungecpp.net/topic/21/tatsoryk-a-lounge-discord-gaem-project-that-gets-completed) [regulars only].

## Elevator Pitch

A 2D top-down pvp arena shooter.

Browser based client and Rust server.

## Using Vagrant

1. Install [VirtualBox](https://www.virtualbox.org/wiki/Downloads) 5.0.14 or newer.
1. Install [Vagrant](https://www.vagrantup.com/downloads.html) 1.8 or newer
1. Run `vagrant up` to set everything up. This needs 1GB free RAM, and will download around 1-2GB (the image + project dependencies).
1. Run `vagrant ssh`. `/vagrant` folder corresponds to the project root.

You should be able to just run builds without installing anything extra. Also, nginx is listening on port 8000 and 8443 (TLS).
WebSocket is exposed as `ws://localhost:8000/ws/` (the server itself should be listening on port 8080). Static files are all
served from `/vagrant/client`.

For Rust there is multirust and newest stable Rust installed.

For C++ side of things there is Clang 3.8, ninja, libwebsocketpp, Boost 1.58 and ICU should Unicode support be needed.
