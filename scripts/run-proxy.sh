#!/bin/sh

# Cleanup
echo "Cleaning up old processes..."
pkill socket-bridge
pkill tcp-bridge

# Ensure IP Alias exists
echo "Setting up IP Alias 127.0.0.2..."
ip addr add 127.0.0.2/8 dev lo 2>/dev/null

# Clean up old iptables rule to avoid duplicates
echo "Cleaning old iptables rules..."
iptables -t nat -D OUTPUT -p tcp -d 127.0.0.1 --dport 2345 ! -s 127.0.0.2 -j DNAT --to-destination 127.0.0.2:9000 2>/dev/null

# Start Proxy in background
echo "Starting TCP Proxy..."
/tmp/tcp-bridge --bind-ip 127.0.0.2 --bind-port 9000 --target-ip 127.0.0.1 --target-port 2345 --source-ip 127.0.0.2 &
PROXY_PID=$!

# Wait for it to start
sleep 1

if ! kill -0 $PROXY_PID 2>/dev/null; then
    echo "Proxy failed to start!"
    exit 1
fi

# Add Redirect Rule
echo "Applying IPTables Redirection..."
iptables -t nat -A OUTPUT -p tcp -d 127.0.0.1 --dport 2345 ! -s 127.0.0.2 -j DNAT --to-destination 127.0.0.2:9000

# Force Reconnection
echo "Killing rc_gui_juce to force new connection..."
pkill rc_gui_juce

echo "Proxy is running (PID $PROXY_PID). Press Ctrl+C to stop."

# Trap Ctrl+C to cleanup
cleanup() {
    echo "Stopping..."
    iptables -t nat -D OUTPUT -p tcp -d 127.0.0.1 --dport 2345 ! -s 127.0.0.2 -j DNAT --to-destination 127.0.0.2:9000 2>/dev/null
    kill $PROXY_PID
}
trap cleanup INT TERM

wait $PROXY_PID
