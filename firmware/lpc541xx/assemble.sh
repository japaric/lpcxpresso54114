#!/bin/bash

set -euxo pipefail

crate=lpc541xx

rm -f bin/*.a

# this will be used by both targets so it needs to be ARMv6-M compatible
arm-none-eabi-as -march=armv6s-m -mthumb -mfloat-abi=soft start.s -o bin/start.o
ar crs bin/thumbv7em-none-eabihf.a bin/start.o

rm bin/*.o
