#!/usr/bin/env bash
[ -z "$2" ] && echo "pass input and output paths" && exit 1
ffmpeg -i "$1" -ar 48000 -ab 128k -ac 1 "$2"
