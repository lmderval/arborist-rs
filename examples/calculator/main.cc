#include <iostream>

#include <arborist/propagate_const_unique_ptr.hh>

#include "all.hh"
#include "cloner.hh"
#include "pretty_printer.hh"

int main() {
    location loc = {};
    std::vector<arborist::propagate_const_unique_ptr<Exp>> exps;
    exps.push_back(arborist::make_propagate_const_unique<BinaryExp>(
        loc, PLUS, arborist::make_propagate_const_unique<IntExp>(loc, 42),
        arborist::make_propagate_const_unique<IntExp>(loc, 21)));
    exps.push_back(arborist::make_propagate_const_unique<IntExp>(loc, 84));
    auto exp = arborist::make_propagate_const_unique<TupleExp>(
        loc, std::move(exps));

    PrettyPrinter printer(std::cout);
    printer.println(*exp);

    auto cloned = Cloner{}.clone(*exp);
    printer.println(*exp);
    printer.println(*cloned);
    return 0;
}
