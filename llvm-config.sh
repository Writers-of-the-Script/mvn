#!/bin/bash

if [ "$1" = "--libs" ]; then
    /usr/bin/llvm-config "$@" "--link-static"
else
    /usr/bin/llvm-config "$@"
fi
