# Makefile — freestanding C11 microkernel for sys1own/shbt-ghost.
# Builds the hosted-test static library consumed by tests/reference_test.c.

CC      ?= gcc
CFLAGS  ?= -O2 -std=c11 -Wall -Wextra -pedantic -DSHBT_HOSTED_TEST -Iinclude
BUILD   := build
OBJ     := $(BUILD)/shbt_ghost_kernel.o
LIB     := $(BUILD)/libshbt_ghost_reference.a

all: $(LIB)

$(BUILD):
	mkdir -p $(BUILD)

$(OBJ): shbt_ghost_kernel.c include/shbt_hardware.h include/shbt_ghost_mmio.h | $(BUILD)
	$(CC) $(CFLAGS) -c $< -o $@

$(LIB): $(OBJ)
	ar rcs $@ $^

clean:
	rm -rf $(BUILD)

.PHONY: all clean
