#!/bin/sh

set -e

git push
cargo build --release
scp target/release/wurstmineberg-graphql wurstmineberg@wurstmineberg.de:/opt/wurstmineberg/bin/wurstmineberg-graphql
ssh wurstmineberg@wurstmineberg.de /opt/wurstmineberg/bin/wurstmineberg-graphql
