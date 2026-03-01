#!/bin/bash

PROFILE="tracy"
PACKAGE="client"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      PROFILE="$2"
      shift 2
      ;;
    --package)
      PACKAGE="$2"
      shift 2
      ;;
    --help)
      echo "Usage: $0 [--profile <profile>] [--package <name>]"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

BEVY_ASSET_ROOT=${PACKAGE} LD_LIBRARY_PATH=target/${PROFILE}/deps:${HOME}/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/lib: rustup run stable  target/${PROFILE}/${PACKAGE}
