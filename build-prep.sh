#!/bin/bash

git submodule update --init
git submodule update --init --recursive --depth=1 wireguard-go-rs

cargo install --git https://github.com/volta-cli/volta && volta setup
sudo apt install -y protobuf-compiler libprotobuf-dev gcc libdbus-1-dev rpm podman
