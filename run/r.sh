if [[ "$1" == "example_projects/"* ]]; then
	python3 -m aim --path `dirname $1`

elif [[ "$1" == *".py" ]]; then
	python3 $1
fi
