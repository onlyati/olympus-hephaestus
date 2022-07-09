publish:
	cd $(shell pwd)/hephaestus && cargo build --release
	sudo cp $(shell pwd)/hephaestus/target/release/hephaestus /usr/share/olympus/hephaestus/
	cd $(shell pwd)/cli && cargo build --release
	sudo cp $(shell pwd)/cli/target/release/cli /usr/share/olympus/hephaestus/

publish_cli:
	cd $(shell pwd)/cli && cargo build --release
	sudo cp $(shell pwd)/cli/target/release/cli /usr/share/olympus/hephaestus/

publish_hephaestus:
	cd $(shell pwd)/hephaestus && cargo build --release
	sudo cp $(shell pwd)/hephaestus/target/release/hephaestus /usr/share/olympus/hephaestus/
