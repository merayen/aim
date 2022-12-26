read_char() {
	stty -icanon -echo
	eval "$1=\$(dd bs=1 count=1 2>/dev/null)"
	stty icanon echo
}

read_char char
bash "run/$char.sh" $1 $2 $3 $4 $5 $6 $7 $8 $9
