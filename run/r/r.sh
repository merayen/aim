export PYTHONPATH="/home/merayen/d/aim"
if [[ "$VIM_FILE" == *"/."*".py" ]]; then
	python3 $VIM_FILE

elif [[ "$VIM_FILE" == "example_projects/"* ]]; then
	python3 -m aim --path `dirname $VIM_FILE`

elif [[ "$VIM_FILE" == *".py" ]]; then
	python3 $VIM_FILE
fi
