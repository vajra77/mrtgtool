
RS_BIN   := bin/mrtgtool
CARGO    := cargo

.PHONY: all


all:
	@echo "===> Compiling Rust ..."
	$(CARGO) build --release
	@cp -f target/release/mrtgtool $@
	@echo "OK: $@"

clean:
	@echo "===> Cleaning target ..."
	@rm -rf $(BIN_DIR)
	$(CARGO) clean 2>/dev/null || true
	@echo "Cleanup completed"
