$(BUILD_DIR)/libcommon.rlib: $(call rwildcard,$(SRC_DIR)common/,*.rs) $(BUILD_DIR)/libcore.rlib $(BUILD_DIR)/libsupport.rlib
	@echo "Building $@"
	@$(RUSTC) $(RUSTC_FLAGS) --out-dir $(BUILD_DIR) $(SRC_DIR)common/lib.rs

