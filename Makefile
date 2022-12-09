
all: 
	@cargo build --release
	@mkdir -p dist
	@cp target/release/imrefs dist/imrefs


install:
	@mkdir -p ~/.local/bin
	@cp dist/imrefs ~/.local/bin

clean:
	@rm -rf dist
	@rm ~/.local/bin/imrefs

