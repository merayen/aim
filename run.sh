#!/usr/bin/env bash
clear
pydex aim > index.txt
python3 -m aim test.txt #&&
#poetry run pylint -E aim.py &&
#poetry run black aim.py
#todo
