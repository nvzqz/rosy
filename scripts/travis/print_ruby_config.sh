#!/usr/bin/env bash

if [[ "$TRAVIS_OS_NAME" != "windows" ]]; then
    RUBY="rvm $ROSY_RUBY_VERSION do ruby"
else
    RUBY="ruby"
fi

$RUBY -e "require 'pp'; pp RbConfig::CONFIG"
