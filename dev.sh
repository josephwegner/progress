#!/bin/bash
# Load .env and run trunk serve with environment variables

if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

trunk serve --release
