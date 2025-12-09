#!/bin/sh
PID=$(ps | grep rc_gui_juce | grep -v grep | awk '{print $1}')
if [ -z "$PID" ]; then
    echo "Process rc_gui_juce not found"
    exit 1
fi
echo "Found rc_gui_juce PID: $PID"
echo "Starting sniffer on all FDs..."
/tmp/socket-sniffer $PID 0
