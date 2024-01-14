TARGET_PATH="/home/merayen/d/Diverse/runs/git"

clear

read_char() {
	stty -icanon -echo
	eval "$1=\$(dd bs=1 count=1 2>/dev/null)"
	stty icanon echo
}

echo -n "$TARGET_PATH>"
read_char char
echo $char

if [[ "$char" = "?" ]]; then
	find $TARGET_PATH -name "*.sh" -exec echo -en "{}	" \; -exec head -n 1 {} \; | sort
	exit
fi

bash "$TARGET_PATH/$char.sh" $1 $2 $3 $4 $5 $6 $7 $8 $9

