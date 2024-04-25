#!/bin/bash
./package_web.sh debug
python3.12 -m http.server 8080 --directory ./out/
