#!/bin/sh

Xephyr -br -ac -noreset -screen 1920x1080 :1 & disown
export DISPLAY=:1
qtile start & disown

