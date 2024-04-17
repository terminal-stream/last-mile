#!/bin/bash
# this builds the docker base image to use as build environment
docker build --tag terminal.stream/build-amazonlinux-openssl-rust:0.1 -f build-amazonlinux-openssl-rust.dockerfile .
