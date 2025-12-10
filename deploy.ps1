$ErrorActionPreference = "Stop"

$Target = "aarch64-unknown-linux-musl"
$RemoteHost = "root@192.168.7.22"
$RemotePath = "/tmp/socket-bridge"
$LocalBin = ".\socket-bridge\target\$Target\release\socket-bridge"

# Build tcp-bridge (includes bridge-ctl and api-server)
Write-Host "Building tcp-bridge, bridge-ctl, and api-server..."
Push-Location "tcp-bridge"
cross build --release --target $Target
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed"
    exit 1
}
Pop-Location

Write-Host "Deploying to $RemoteHost..."
# Kill existing instances to release file lock
ssh $RemoteHost "pkill tcp-bridge || true; pkill api-server || true"

# Deploy
scp -O `
    ./tcp-bridge/target/$Target/release/tcp-bridge `
    ./tcp-bridge/target/$Target/release/bridge-ctl `
    ./tcp-bridge/target/$Target/release/api-server `
    ./scripts/run-sniffer.sh `
    ./scripts/run-proxy.sh `
    ./scripts/test_mappings.sh `
    ./scripts/run-api.sh `
    "$($RemoteHost):/tmp/"
ssh $RemoteHost "chmod +x  /tmp/bridge-ctl /tmp/api-server /tmp/run-sniffer.sh /tmp/tcp-bridge /tmp/run-proxy.sh /tmp/test_mappings.sh /tmp/run-api.sh"

Write-Host "Done."
