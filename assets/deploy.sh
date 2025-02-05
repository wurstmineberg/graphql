#!/bin/sh

set -e

git push
cargo build --release
ssh wurstmineberg.de sudo systemctl stop wmb-graphql
scp target/release/wurstmineberg-graphql wurstmineberg@wurstmineberg.de:/opt/wurstmineberg/bin/wurstmineberg-graphql
ssh wurstmineberg.de sudo systemctl start wmb-graphql
