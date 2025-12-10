#!/bin/sh
# Start the API server
# Assumes tcp-bridge proxy is already running

PORT=${1:-8080}

echo "Starting API server on port $PORT..."
/tmp/api-server &
echo "API server started with PID $!"
