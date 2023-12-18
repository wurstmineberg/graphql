function ThrowOnNativeFailure {
    if (-not $?)
    {
        throw 'Native Failure'
    }
}

wsl cargo build --release
ThrowOnNativeFailure

scp .\target\release\wurstmineberg-graphql wurstmineberg@wurstmineberg.de:/opt/wurstmineberg/bin/wurstmineberg-graphql
ThrowOnNativeFailure

ssh wurstmineberg@wurstmineberg.de /opt/wurstmineberg/bin/wurstmineberg-graphql
ThrowOnNativeFailure
