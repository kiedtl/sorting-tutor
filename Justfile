build:
	clear
	trunk build
	zola -r site build

serve: build
	zola -r site serve
