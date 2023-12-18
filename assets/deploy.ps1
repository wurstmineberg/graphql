function ThrowOnNativeFailure {
    if (-not $?)
    {
        throw 'Native Failure'
    }
}

git push
ThrowOnNativeFailure

wsl --distribution debian-m2 cargo build --release
ThrowOnNativeFailure

scp .\target\release\wurstmineberg-graphql wurstmineberg@wurstmineberg.de:/opt/wurstmineberg/bin/wurstmineberg-graphql
ThrowOnNativeFailure

ssh wurstmineberg@wurstmineberg.de /opt/wurstmineberg/bin/wurstmineberg-graphql
ThrowOnNativeFailure
