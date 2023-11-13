#!/bin/bash
# this builds the base docker image to use as execution environment
docker build --tag terminal.stream/exec-amazonlinux-openssl:0.1 -f exec-amazonlinux-openssl.dockerfile .
