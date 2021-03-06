# userland master makefile. Included by application makefiles

TOCK_USERLAND_BASE_DIR ?= $(abspath .)
TOCK_BASE_DIR ?= $(abspath ../)
BUILDDIR ?= $(abspath .)
TOCK_BOARD ?= storm
TOCK_ARCH ?= cortex-m4
LIBTOCK ?= $(TOCK_USERLAND_BASE_DIR)/libtock/build/$(TOCK_ARCH)/libtock.a

TOOLCHAIN := arm-none-eabi

# This could be replaced with an installed version of `elf2tbf`
ELF2TBF ?= cargo run --manifest-path $(TOCK_USERLAND_BASE_DIR)/tools/elf2tbf/Cargo.toml --

AS := $(TOOLCHAIN)-as
ASFLAGS += -mcpu=$(TOCK_ARCH) -mthumb

CC := $(TOOLCHAIN)-gcc
CXX := $(TOOLCHAIN)-g++
# n.b. make convention is that CPPFLAGS are shared for C and C++ sources
# [CFLAGS is C only, CXXFLAGS is C++ only]
CFLAGS   += -std=gnu11
CPPFLAGS += -I$(TOCK_USERLAND_BASE_DIR)/libtock -g -mcpu=$(TOCK_ARCH) -mthumb -mfloat-abi=soft
CPPFLAGS += \
	    -fdata-sections -ffunction-sections\
	    -Wall\
	    -Wextra\
	    -Wl,-gc-sections\
	    -g\
	    -fPIC\
	    -msingle-pic-base\
	    -mpic-register=r9\
	    -mno-pic-data-is-text-relative

# First step doesn't actually compile, just generate header dependency information
# More info on our approach here: http://stackoverflow.com/questions/97338
$(BUILDDIR)/%.o: %.c | $(BUILDDIR)
	$(TRACE_DEP)
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS) -MF"$(@:.o=.d)" -MG -MM -MP -MT"$(@:.o=.d)@" -MT"$@" "$<"
	$(TRACE_CC)
	$(Q)$(CC) $(CFLAGS) $(CPPFLAGS) -c -o $@ $<

LINKER ?= $(TOCK_USERLAND_BASE_DIR)/linker.ld

SIZE := $(TOOLCHAIN)-size

.PHONY:	all
all:	$(BUILDDIR)/app.bin size

.PHONY: size
size:	$(BUILDDIR)/app.elf
	@$(SIZE) $<

# Include the libtock makefile. Adds rules that will rebuild library when needed
include $(TOCK_USERLAND_BASE_DIR)/libtock/Makefile

$(BUILDDIR):
	$(Q)mkdir -p $(BUILDDIR)

$(BUILDDIR)/app.elf: $(OBJS) $(TOCK_USERLAND_BASE_DIR)/newlib/libc.a $(LIBTOCK) | $(BUILDDIR)
	$(TRACE_LD)
	$(Q)$(CC) -Wl,--gc-sections -Wl,--emit-relocs --entry=_start $(CFLAGS) $(CPPFLAGS) -T $(LINKER) -nostdlib $(OBJS) -Wl,--start-group $(TOCK_USERLAND_BASE_DIR)/newlib/libc.a $(LIBTOCK) -lm -lgcc -Wl,--end-group -o $@

$(BUILDDIR)/app.bin: $(BUILDDIR)/app.elf | $(BUILDDIR)
	$(TRACE_BIN)
	$(Q)$(ELF2TBF) -o $@ $<

# for programming individual apps, include platform app makefile
#	conditionally included in case it doesn't exist for a board
-include $(TOCK_BASE_DIR)/boards/$(TOCK_BOARD)/Makefile-app



#########################################################################################
## Pretty-printing rules

# If environment variable V is non-empty, be verbose
ifneq ($(V),)
Q=
TRACE_BIN =
TRACE_DEP =
TRACE_CC  =
TRACE_CXX =
TRACE_LD  =
TRACE_AR  =
TRACE_AS  =
TRACE_LST =
else
Q=@
TRACE_BIN = @echo " BIN       " $@
TRACE_DEP = @echo " DEP       " $<
TRACE_CC  = @echo "  CC       " $<
TRACE_CXX = @echo " CXX       " $<
TRACE_LD  = @echo "  LD       " $@
TRACE_AR  = @echo "  AR       " $@
TRACE_AS  = @echo "  AS       " $<
TRACE_LST = @echo " LST       " $<
endif



#########################################################################################
# Include dependency rules for picking up header changes (by convention at bottom of makefile)
-include $(OBJS:.o=.d)
