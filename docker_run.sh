#!/bin/bash
# Run interactive exposing all ports configured, uses the local config at ./config.
docker run --rm --detach --name last-mile -P -v $(pwd)/config:/usr/local/share/tslm-config terminal.stream/last-mile:0.1 --config-dir=/usr/local/share/tslm-config
