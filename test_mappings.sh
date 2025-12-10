#!/bin/sh
# Interactive Fader/Source Mapping Test
# Usage: ./test_mappings.sh [f_start] [f_end] [s_start] [s_end]

F_START=${1:-0}
F_END=${2:-8}
S_START=${3:-0}
S_END=${4:-16}

echo "Starting Interactive Mapping Test..."
echo "Faders: $F_START to $F_END"
echo "Sources: $S_START to $S_END"
echo "You will be prompted before each change."
echo "Press Ctrl+C to exit at any time."

# Iterate Fader Indices
for f in $(seq $F_START $F_END); do
    echo ""
    echo "########################################"
    echo "# Switching to Fader Index: $f"
    echo "########################################"
    
    # Iterate Source IDs
    for s in $(seq $S_START $S_END); do
        echo ""
        echo "----------------------------------------"
        echo "Preparing: Fader $f -> Source $s"
        echo "Current Source ID: $s"
        printf "Press [Enter] to APPLY, or 's' then [Enter] to SKIP this Source: "
        read input
        
        if [ "$input" = "s" ]; then
            echo "Skipping Source $s..."
            continue
        fi
        
        echo "Applying..."
        
        # 1. Set Source
        /tmp/bridge-ctl source $f $s
        
        # 2. Fix UI Consistency (MicType Sequence)
        # Assuming this is required to prevent UI corruption
        /tmp/bridge-ctl mic_type $f -1
        /tmp/bridge-ctl mic_type $f 4
        
        echo "Done. Check UI."
    done
done

echo "Test Complete."
