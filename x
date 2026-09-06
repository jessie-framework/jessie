if command -v python >/dev/null 2>&1; then
	python ./x.py "$@"
else
	python3 ./x.py "$@"
fi
