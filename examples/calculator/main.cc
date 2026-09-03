#include <iostream>

#include <arborist/propagate_const_unique_ptr.hh>

#include "all.hh"
#include "pretty_printer.hh"

int main() {
    location loc = {};
    auto exp = arborist::make_propagate_const_unique<BinaryExp>(
        loc, PLUS, arborist::make_propagate_const_unique<IntExp>(loc, 42),
        arborist::make_propagate_const_unique<IntExp>(loc, 21));

    PrettyPrinter printer(std::cout);
    printer.println(*exp);
    return 0;
}
