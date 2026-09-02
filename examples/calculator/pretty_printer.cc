#include "pretty_printer.hh"

#include "all.hh"

namespace {
    const char* operators[] = {
        [PLUS] = "+",
        [MINUS] = "-",
    };
}

PrettyPrinter::PrettyPrinter(std::ostream& os)
    : os_(os) {}

void PrettyPrinter::operator()(const IntExp& e) { os_ << e.value_get(); }

void PrettyPrinter::operator()(const UnaryExp& e) {
    os_ << operators[e.op_get()];
    e.exp_get().accept(*this);
}

void PrettyPrinter::operator()(const BinaryExp& e) {
    e.lhs_get().accept(*this);
    os_ << operators[e.op_get()];
    e.rhs_get().accept(*this);
}

void PrettyPrinter::print(const Exp& e) {
    e.accept(*this);
}

void PrettyPrinter::println(const Exp& e) {
    e.accept(*this);
    os_ << std::endl;
}
