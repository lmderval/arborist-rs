#include <cassert>
#include <memory>
#include <type_traits>

#include <arborist/propagate_const_unique_ptr.hh>
#include <gtest/gtest.h>

namespace {
    struct Base {
        int value;

        explicit Base(int v = 0)
            : value(v) {}
        virtual ~Base() = default;

        int get_value() const { return value; }
        void set_value(int v) { value = v; }
    };

    struct Derived : public Base {
        int extra;

        Derived(int v, int e)
            : Base(v)
            , extra(e) {}
    };

    struct CustomDeleter {
        bool* deleted_flag = nullptr;

        void operator()(Base* p) const {
            if (deleted_flag) { *deleted_flag = true; }
            delete p;
        }
    };
} // namespace

// ============================================================================
// Compile-time static assertions (const propagation and types)
// ============================================================================

// 1. arborist::propagate_const_unique_ptr deep-const propagation
static_assert(std::is_same_v<
              decltype(*std::declval<
                       const arborist::propagate_const_unique_ptr<Base>&>()),
              const Base&>);
static_assert(
    std::is_same_v<
        decltype(*std::declval<arborist::propagate_const_unique_ptr<Base>&>()),
        Base&>);
static_assert(std::is_same_v<
              decltype(std::declval<
                           const arborist::propagate_const_unique_ptr<Base>&>()
                           .get()),
              const Base*>);
static_assert(
    std::is_same_v<
        decltype(std::declval<arborist::propagate_const_unique_ptr<Base>&>()
                     .get()),
        Base*>);
static_assert(std::is_same_v<
              decltype(std::declval<
                           const arborist::propagate_const_unique_ptr<Base>&>()
                           .operator->()),
              const Base*>);
static_assert(
    std::is_same_v<
        decltype(std::declval<arborist::propagate_const_unique_ptr<Base>&>()
                     .operator->()),
        Base*>);

// 2. Contrast with std::unique_ptr:
// With std::unique_ptr, const dereferencing STILL yields a mutable reference:
static_assert(std::is_convertible_v<
              decltype(*std::declval<const std::unique_ptr<Base>&>()), Base&>);
// With arborist::propagate_const_unique_ptr, const dereferencing DISALLOWS
// obtaining a mutable reference:
static_assert(!std::is_convertible_v<
              decltype(*std::declval<
                       const arborist::propagate_const_unique_ptr<Base>&>()),
              Base&>);
// Non-const arborist::propagate_const_unique_ptr allows obtaining a mutable
// reference:
static_assert(
    std::is_convertible_v<
        decltype(*std::declval<arborist::propagate_const_unique_ptr<Base>&>()),
        Base&>);

// 3. Custom Deleter support
static_assert(
    std::is_same_v<
        arborist::propagate_const_unique_ptr<Base, CustomDeleter>::deleter_type,
        CustomDeleter>);

// ============================================================================
// Google Test runtime tests
// ============================================================================

TEST(NodePtrTest, ConstPropagation) {
    auto ptr = arborist::make_propagate_const_unique<Base>(42);
    EXPECT_EQ(ptr->get_value(), 42);

    // Mutation through non-const arborist::propagate_const_unique_ptr
    ptr->set_value(100);
    EXPECT_EQ(ptr->get_value(), 100);
    (*ptr).set_value(200);
    EXPECT_EQ(ptr->get_value(), 200);

    // Deep constness through const reference:
    const arborist::propagate_const_unique_ptr<Base>& const_ref = ptr;
    EXPECT_EQ(const_ref->get_value(), 200);
    EXPECT_EQ((*const_ref).get_value(), 200);
    EXPECT_EQ(const_ref.get()->get_value(), 200);

    // Verify pointer types at runtime
    const Base* raw_const = const_ref.get();
    EXPECT_EQ(raw_const->get_value(), 200);
}

TEST(NodePtrTest, DerivedToBaseConversion) {
    // Converting move constructor
    arborist::propagate_const_unique_ptr<Derived> d =
        arborist::make_propagate_const_unique<Derived>(10, 20);
    EXPECT_EQ(d->extra, 20);

    arborist::propagate_const_unique_ptr<Base> b = std::move(d);
    EXPECT_EQ(d, nullptr);
    ASSERT_NE(b, nullptr);
    EXPECT_EQ(b->get_value(), 10);

    // Converting move assignment
    arborist::propagate_const_unique_ptr<Derived> d2 =
        arborist::make_propagate_const_unique<Derived>(30, 40);
    arborist::propagate_const_unique_ptr<Base> b2;
    b2 = std::move(d2);
    EXPECT_EQ(d2, nullptr);
    ASSERT_NE(b2, nullptr);
    EXPECT_EQ(b2->get_value(), 30);
}

TEST(NodePtrTest, UniquePtrInterop) {
    // Construct from std::unique_ptr (including derived-to-base)
    auto u_derived = std::make_unique<Derived>(50, 60);
    arborist::propagate_const_unique_ptr<Base> b = std::move(u_derived);
    ASSERT_NE(b, nullptr);
    EXPECT_EQ(b->get_value(), 50);

    // Assign from std::unique_ptr
    auto u_derived2 = std::make_unique<Derived>(70, 80);
    arborist::propagate_const_unique_ptr<Base> b2;
    b2 = std::move(u_derived2);
    ASSERT_NE(b2, nullptr);
    EXPECT_EQ(b2->get_value(), 70);

    // Convert back to std::unique_ptr via to_unique_ptr() &&
    std::unique_ptr<Base> u_back = std::move(b2).to_unique_ptr();
    ASSERT_NE(u_back, nullptr);
    EXPECT_EQ(u_back->get_value(), 70);
    EXPECT_EQ(b2, nullptr);
}

TEST(NodePtrTest, ModifiersAndComparisons) {
    arborist::propagate_const_unique_ptr<Base> p1 =
        arborist::make_propagate_const_unique<Base>(1);
    arborist::propagate_const_unique_ptr<Base> p2 =
        arborist::make_propagate_const_unique<Base>(2);

    EXPECT_TRUE(static_cast<bool>(p1));
    EXPECT_NE(p1, nullptr);
    EXPECT_NE(nullptr, p1);
    EXPECT_NE(p1, p2);

    // swap
    arborist::swap(p1, p2);
    EXPECT_EQ(p1->get_value(), 2);
    EXPECT_EQ(p2->get_value(), 1);

    // reset
    p1.reset();
    EXPECT_FALSE(static_cast<bool>(p1));
    EXPECT_EQ(p1, nullptr);
    EXPECT_EQ(nullptr, p1);

    // release
    Base* raw = p2.release();
    EXPECT_EQ(p2, nullptr);
    ASSERT_NE(raw, nullptr);
    EXPECT_EQ(raw->get_value(), 1);
    delete raw;
}

TEST(NodePtrTest, CustomDeleter) {
    bool was_deleted = false;
    {
        CustomDeleter del{ &was_deleted };
        arborist::propagate_const_unique_ptr<Base, CustomDeleter> p(
            new Base(99), del);
        EXPECT_EQ(p->get_value(), 99);
        EXPECT_FALSE(was_deleted);
    }
    EXPECT_TRUE(was_deleted);
}
