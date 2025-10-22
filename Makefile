
all: xml json

xml:
	tidy -iqm -wrap 0 -xml splits/*/*.lss splits/*/*.lsl

json:
	for i in splits/*/*.ls1l; do \
	    jq --indent 4 . "$$i" > "$${i}.tmp" && mv "$${i}.tmp" "$$i" ; \
	done

examples/splits.json: src/splits.rs examples/splits.rs
	cargo run --example splits --target $$(rustc -vV | sed -n 's|host: ||p')

fmt:
	cargo fmt

clippy:
	cargo clippy --all-features -- -A clippy::nonminimal_bool

clean:
	rm splits/*/*.tmp
