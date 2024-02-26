#!/bin/sh

bindgen \
  --output ./src/bindings.rs \
  src/wrapper.h
