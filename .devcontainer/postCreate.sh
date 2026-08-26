#!/usr/bin/env bash

rustc --version

rustup toolchain add nightly
rustup component add --toolchain nightly rustfmt

# the named volume mounts as root, pnpm runs as the remote user
sudo chown "$(id --user):$(id --group)" node_modules

pnpm install
