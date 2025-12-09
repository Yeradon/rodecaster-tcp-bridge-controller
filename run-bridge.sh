#!/bin/sh
# Find PID of rc_gui_juce
PID=$(pidof rc_gui_juce)
if [ -z "$PID" ]; then
    echo "rc_gui_juce not found!"
    exit 1
fi

echo "Found rc_gui_juce PID: $PID"

# Check if already traced
if grep -q "TracerPid:[[:space:]]*[1-9]" /proc/$PID/status; then
    TRACER=$(grep TracerPid /proc/$PID/status | awk '{print $2}')
    echo "Error: Process $PID is already being traced by PID $TRACER"
    CMD_NAME=$(ps -p $TRACER -o comm=)
    echo "Tracer name: $CMD_NAME"
    
    if [ "$CMD_NAME" = "socket-bridge" ]; then
        echo "Killing stale socket-bridge (PID $TRACER)..."
        kill -9 $TRACER
        sleep 1
    else
        echo "Please kill the tracer manually."
        exit 1
    fi
fi

echo "Starting Socket Bridge on FD 10..."
/tmp/socket-bridge $PID 10
