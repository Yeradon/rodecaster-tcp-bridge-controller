$ErrorActionPreference = "Stop"

$Target = "aarch64-unknown-linux-musl"
$RemoteHost = "root@192.168.7.22"
$RemotePath = "/tmp/socket-bridge"
$LocalBin = ".\socket-bridge\target\$Target\release\socket-bridge"

Write-Host "Building for $Target..."
Push-Location socket-bridge
# You might need 'cross' if you don't have the target installed directly
# cargo build --release --target $Target
cross build --release --target $Target
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed"
}
Pop-Location

Write-Host "Deploying to $RemoteHost..."
# Added -v for verbose to debug
scp -v $LocalBin "$RemoteHost`:$RemotePath"

Write-Host "Done."
