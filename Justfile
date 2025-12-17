build:
	clear
	trunk build
	zola -r site build

serve: build
	zola -r site serve

release:
	clear
	trunk build --release
	zola -r site build
	rm -rf dist
	cp -r site/public dist
	for file in $(find site/public -name '*html'); do minify "$file" -o dist/; done
