CARGO := cargo

test:
	@$(CARGO) test -q

clippy:
	@$(CARGO) clippy -q --color=always
