if [[ "$VIM_FILE" = "commands.txt" ]]; then
	clear
	bash <(sed "${VIM_LINENO}q;d" $VIM_FILE)
else
	quickcommand r
fi
