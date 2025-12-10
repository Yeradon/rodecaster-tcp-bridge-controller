#!/bin/sh
# Finds PID of rc_gui_juce and runs sniffer
PID=$(pgrep rc_gui_juce)

if [ -z "$PID" ]; then
    # Fallback to ps if pgrep missing
    PID=$(ps | grep rc_gui_juce | grep -v grep | awk '{print $1}')
fi

if [ -z "$PID" ]; then
    echo "Error: Process rc_gui_juce not found."
    exit 1
fi

echo "Found rc_gui_juce PID: $PID"
echo "Starting sniffer on all FDs..."
/tmp/socket-sniffer $PID 0
