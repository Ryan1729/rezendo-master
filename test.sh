#!/bin/bash

# copying the bearlib geometry primitives into `common` seems to
# cause an error while testing. Currently only
# `state_manipulation` has tests worth running.

cargo test -p state_manipulation
