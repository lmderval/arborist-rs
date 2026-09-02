#include <iostream>

#include "all.hh"
#include "pretty_printer.hh"

int main() {
    location loc = {};
    auto exp = std::make_unique<BinaryExp>(loc, PLUS,
                                           std::make_unique<IntExp>(loc, 42),
                                           std::make_unique<IntExp>(loc, 21));
    PrettyPrinter printer(std::cout);
    printer.println(*exp);
    return 0;
}
