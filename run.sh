if [ "$1" == "plan.md" ]; then
	plan
	exit
fi

read_char() {
	stty -icanon -echo
	eval "$1=\$(dd bs=1 count=1 2>/dev/null)"
	stty icanon echo
}

echo -n ">"
read_char char

if [[ "$char" = "?" ]]; then
	find . -name "*.sh" -exec basename {} \; -exec head -n 1 {} \; -exec echo \;
	exit
fi

bash "run/$char.sh" $1 $2 $3 $4 $5 $6 $7 $8 $9
