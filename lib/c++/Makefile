CXX = clang++
CPPFLAGS = -Iinclude $(shell pkg-config --cflags gtest gtest_main)
CXXFLAGS = -std=c++23 -Werror -Wall -Wextra -pedantic
LDLIBS = $(shell pkg-config --libs gtest gtest_main)

SRC = \
	test/test_propagate_const_unique_ptr.cc

BIN = $(SRC:.cc=)

all: $(BIN)

check: $(BIN)
	$(foreach bin,$(BIN),./$(bin);)

clean:
	$(RM) $(BIN)

.PHONY: all check clean
