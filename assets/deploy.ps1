git push
if (-not $?)
{
    throw 'Native Failure'
}

# copy the tree to the WSL file system to improve compile times
wsl --distribution debian-m2 rsync --delete -av /mnt/c/Users/fenhl/git/github.com/wurstmineberg/graphql/stage/ /home/fenhl/wslgit/github.com/wurstmineberg/graphql/ --exclude target
if (-not $?)
{
    throw 'Native Failure'
}

wsl --distribution debian-m2 env -C /home/fenhl/wslgit/github.com/wurstmineberg/graphql cargo build --release --target=x86_64-unknown-linux-musl
if (-not $?)
{
    throw 'Native Failure'
}

wsl --distribution debian-m2 mkdir -p /mnt/c/Users/fenhl/git/github.com/wurstmineberg/graphql/stage/target/wsl/release
if (-not $?)
{
    throw 'Native Failure'
}

wsl --distribution debian-m2 cp /home/fenhl/wslgit/github.com/wurstmineberg/graphql/target/x86_64-unknown-linux-musl/release/wurstmineberg-graphql /mnt/c/Users/fenhl/git/github.com/wurstmineberg/graphql/stage/target/wsl/release/wurstmineberg-graphql
if (-not $?)
{
    throw 'Native Failure'
}

ssh wurstmineberg.de sudo systemctl stop wmb-graphql
if (-not $?)
{
    throw 'Native Failure'
}

scp .\target\wsl\release\wurstmineberg-graphql wurstmineberg@wurstmineberg.de:/opt/wurstmineberg/bin/wurstmineberg-graphql
if (-not $?)
{
    throw 'Native Failure'
}

ssh wurstmineberg.de sudo systemctl start wmb-graphql
if (-not $?)
{
    throw 'Native Failure'
}
