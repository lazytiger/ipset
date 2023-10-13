#!/bin/bash

bindgen wrapper.h -o src/binding.rs
rustfmt src/binding.rs