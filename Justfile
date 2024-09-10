# Common project tasks

tags:
	ctags -R --exclude=*/*.json --exclude=target/* .

lines:
	pygount --format=summary --folders-to-skip=target,data,__pycache__,.git --names-to-skip=tags,*.html



