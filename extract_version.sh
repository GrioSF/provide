#!/bin/bash
cat Cargo.toml | grep '^version = .*\"$' | sed 's/\"//g' | awk '{print $3}'