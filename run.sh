#!/usr/bin/env bash
clear
pydex aim > index.txt &&
/usr/bin/time -v -o benchmark.txt python3 -m aim main.md #&&
echo
grep Elapsed benchmark.txt
#poetry run pylint -E aim.py &&
#poetry run black aim.py
#todo
