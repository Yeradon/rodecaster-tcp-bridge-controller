$ErrorActionPreference = "Stop"

$Target = "aarch64-unknown-linux-musl"
$RemoteHost = "root@192.168.7.22"
$RemotePath = "/tmp/socket-bridge"
$LocalBin = ".\socket-bridge\target\$Target\release\socket-bridge"

# Build tcp-bridge (includes bridge-ctl)
Write-Host "Building tcp-bridge and bridge-ctl..."
Push-Location "tcp-bridge"
cross build --release --target $Target
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed"
    exit 1
}
Pop-Location

Write-Host "Deploying to $RemoteHost..."
# Kill existing instance to release file lock
ssh $RemoteHost "pkill tcp-bridge || true"

# Deploy
scp -O `
    ./tcp-bridge/target/$Target/release/tcp-bridge `
    ./tcp-bridge/target/$Target/release/bridge-ctl `
    ./run-sniffer.sh `
    ./run-proxy.sh `
    ./test_mappings.sh `
    "$($RemoteHost):/tmp/"
ssh $RemoteHost "chmod +x $RemotePath /tmp/bridge-ctl /tmp/run-sniffer.sh /tmp/tcp-bridge /tmp/run-proxy.sh /tmp/test_mappings.sh"

Write-Host "Done."
