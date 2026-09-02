CXX = clang++
CXXFLAGS = -std=c++23 -Werror -Wall -Wextra -pedantic -Wno-c99-designator

ARBORIST = TEMPLATES_DIR=../../templates ../../target/debug/arborist-rs
ARBOCONF = config.arb

FORMAT = clang-format -i

ARBOGEN = \
	all.hh \
	fwd.hh \
	visitor.cc \
	visitor.hh

ARBONODE = \
	exp.cc \
	exp.hh \
	int_exp.cc \
	int_exp.hh \
	unary_exp.cc \
	unary_exp.hh \
	binary_exp.cc \
	binary_exp.hh

GENERATED = \
	$(ARBOGEN) \
	$(ARBONODE)

SRC = \
	exp.cc \
	int_exp.cc \
	unary_exp.cc \
	binary_exp.cc \
	visitor.cc \
	pretty_printer.cc \
	main.cc

OBJ = $(SRC:.cc=.o)

BIN = calculator

all: arbogen $(BIN)

arbogen: $(ARBOCONF) $(ARBOGEN)

%.cc: %.cc.jinja $(ARBOCONF)
	$(RM) $@
	$(ARBORIST) --config $(ARBOCONF) render --template $< --in-place
	$(FORMAT) $@
	chmod -w $@

%: %.jinja $(ARBOCONF)
	$(RM) $@
	$(ARBORIST) --config $(ARBOCONF) render --template $< --in-place
	$(FORMAT) $@
	chmod -w $@

$(ARBONODE)&: node.cc.jinja node.hh.jinja $(ARBOCONF)
	$(RM) $(ARBONODE)
	$(ARBORIST) --config $(ARBOCONF) render --in-place
	$(FORMAT) $(ARBONODE)
	chmod -w $(ARBONODE)

$(BIN): $(OBJ)
	$(CXX) $(LDFLAGS) -o $@ $^ $(LDLIBS)

clean:
	$(RM) $(BIN)
	$(RM) $(OBJ)
	$(RM) $(GENERATED)

.PHONY: all arbogen clean
