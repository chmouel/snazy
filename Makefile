CARGO := cargo

test:
	@$(CARGO) test -q
.PHONY: test

clippy:
	@$(CARGO) clippy -q --color=always
.PHONY: clippy

lint: clippy
.PHONY: lint

lint-fix:
	@$(CARGO) clippy -q --color=always --fix

coverage:
	@$(CARGO) tarpaulin --out=Html --output-dir /tmp/cov-output && \
		type -p open && cmd=open || type -p xdg-open && cmd=xdg-open; \
		$$cmd /tmp/cov-output/tarpaulin-report.html
.PHONY: coverage
