#!/bin/bash

# Extract a Stellaris save file (.sav is a zip archive)
# Usage: ./prepare_data.sh 2337.02.02-testing
# Extracts gamestate and meta from $1.sav into $1/

unzip -o "$1.sav" -d "$1"
