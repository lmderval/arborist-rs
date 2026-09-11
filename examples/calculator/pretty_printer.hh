#pragma once

#include <ostream>

#include "visitor.hh"

class PrettyPrinter : public ConstVisitor {
public:
    PrettyPrinter(std::ostream& os);

public:
    void operator()(const IntExp& e) override;
    void operator()(const UnaryExp& e) override;
    void operator()(const BinaryExp& e) override;
    void operator()(const TupleExp& e) override;

    void print(const Exp& e);
    void println(const Exp& e);

private:
    std::ostream& os_;
};
