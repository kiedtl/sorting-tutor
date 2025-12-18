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
	for file in $(find site/public -name '*html'); do minify "$file" -o "dist/${file##site/public/}"; done

publish:
	ssh team 'rm -rf ~/src/sorting-tutor/dist'
	scp -r dist team:~kiedtl/src/sorting-tutor/dist
