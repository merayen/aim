PYTHONMALLOC=debug
PYTHONASYNCIODEBUG=1
python3 -W default -X faulthandler $1 > $1.out
