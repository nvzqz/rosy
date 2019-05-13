#!/usr/bin/env bash

function error() {
    >&2 echo "$@"
    exit 1
}

# `rvm` is not available on Windows
[[ "$TRAVIS_OS_NAME" != "windows" ]] || exit 0

[[ -n "$ROSY_RUBY_VERSION" ]] || error "Specify Ruby version via 'ROSY_RUBY_VERSION'"

if [[ "$FEATURES" == *"static"* ]]; then
    echo "Setting up Ruby $ROSY_RUBY_VERSION for static linking..."
    CONFIGURE_OPTS="--disable-shared"
else
    echo "Setting up Ruby $ROSY_RUBY_VERSION for shared linking..."
    CONFIGURE_OPTS="--enable-shared"
fi

rvm install "$ROSY_RUBY_VERSION" --no-docs "$CONFIGURE_OPTS"
