/**
 * \file propagate_const_unique_ptr.hh
 * \brief Definition of arborist::propagate_const_unique_ptr, a
 * const-propagating unique smart pointer.
 */

#pragma once

#include <concepts>
#include <memory>
#include <type_traits>
#include <utility>

namespace arborist {
    template <typename T, typename Deleter = std::default_delete<T>>
    class propagate_const_unique_ptr {
    public:
        using element_type = T;
        using deleter_type = Deleter;
        using pointer = typename std::unique_ptr<T, Deleter>::pointer;

        // Constructors
        constexpr propagate_const_unique_ptr() noexcept
            requires std::is_default_constructible_v<Deleter>
            && (!std::is_pointer_v<Deleter>)
            : ptr_(nullptr) {}

        constexpr propagate_const_unique_ptr(std::nullptr_t) noexcept
            requires std::is_default_constructible_v<Deleter>
            && (!std::is_pointer_v<Deleter>)
            : ptr_(nullptr) {}

        explicit propagate_const_unique_ptr(pointer ptr) noexcept
            requires std::is_default_constructible_v<Deleter>
            && (!std::is_pointer_v<Deleter>)
            : ptr_(ptr) {}

        propagate_const_unique_ptr(pointer ptr, const Deleter& d) noexcept
            requires std::is_copy_constructible_v<Deleter>
            : ptr_(ptr, d) {}

        propagate_const_unique_ptr(pointer ptr, Deleter&& d) noexcept
            requires std::is_move_constructible_v<Deleter>
            : ptr_(ptr, std::move(d)) {}

        // Interop with std::unique_ptr
        template <typename U, typename D>
            requires std::convertible_to<
                         typename std::unique_ptr<U, D>::pointer, pointer>
            && std::convertible_to<D, Deleter> && (!std::is_array_v<U>)
        /* implicit */ propagate_const_unique_ptr(
            std::unique_ptr<U, D>&& u) noexcept
            : ptr_(std::move(u)) {}

        // Move semantics
        propagate_const_unique_ptr(
            propagate_const_unique_ptr&& other) noexcept = default;
        propagate_const_unique_ptr&
        operator=(propagate_const_unique_ptr&& other) noexcept = default;

        // Converting move constructor & assignment for derived-to-base
        template <typename U, typename D>
            requires std::convertible_to<
                         typename std::unique_ptr<U, D>::pointer, pointer>
            && std::convertible_to<D, Deleter> && (!std::is_array_v<U>)
        propagate_const_unique_ptr(
            propagate_const_unique_ptr<U, D>&& other) noexcept
            : ptr_(std::move(other.ptr_)) {}

        template <typename U, typename D>
            requires std::convertible_to<
                         typename std::unique_ptr<U, D>::pointer, pointer>
            && std::convertible_to<D, Deleter> && (!std::is_array_v<U>)
        propagate_const_unique_ptr&
        operator=(propagate_const_unique_ptr<U, D>&& other) noexcept {
            ptr_ = std::move(other.ptr_);
            return *this;
        }

        template <typename U, typename D>
            requires std::convertible_to<
                         typename std::unique_ptr<U, D>::pointer, pointer>
            && std::convertible_to<D, Deleter> && (!std::is_array_v<U>)
        propagate_const_unique_ptr&
        operator=(std::unique_ptr<U, D>&& other) noexcept {
            ptr_ = std::move(other);
            return *this;
        }

        propagate_const_unique_ptr& operator=(std::nullptr_t) noexcept {
            reset();
            return *this;
        }

        // Delete copy
        propagate_const_unique_ptr(const propagate_const_unique_ptr&) = delete;
        propagate_const_unique_ptr&
        operator=(const propagate_const_unique_ptr&) = delete;

        ~propagate_const_unique_ptr() = default;

        // Observers: Deep const propagation
        [[nodiscard]] constexpr pointer get() noexcept { return ptr_.get(); }

        [[nodiscard]] constexpr std::add_const_t<element_type>*
        get() const noexcept {
            return ptr_.get();
        }

        [[nodiscard]] constexpr element_type& operator*() noexcept {
            return *ptr_;
        }

        [[nodiscard]] constexpr const element_type& operator*() const noexcept {
            return *ptr_;
        }

        [[nodiscard]] constexpr pointer operator->() noexcept {
            return ptr_.get();
        }

        [[nodiscard]] constexpr std::add_const_t<element_type>*
        operator->() const noexcept {
            return ptr_.get();
        }

        [[nodiscard]] explicit constexpr operator bool() const noexcept {
            return static_cast<bool>(ptr_);
        }

        // Deleter access
        [[nodiscard]] Deleter& get_deleter() noexcept {
            return ptr_.get_deleter();
        }

        [[nodiscard]] const Deleter& get_deleter() const noexcept {
            return ptr_.get_deleter();
        }

        // Modifiers
        pointer release() noexcept { return ptr_.release(); }

        void reset(pointer p = pointer()) noexcept { ptr_.reset(p); }

        void swap(propagate_const_unique_ptr& other) noexcept {
            ptr_.swap(other.ptr_);
        }

        // Conversion to std::unique_ptr (rvalue only, transfers ownership)
        [[nodiscard]] std::unique_ptr<T, Deleter> to_unique_ptr() && noexcept {
            return std::move(ptr_);
        }

        // Comparisons with nullptr
        friend constexpr bool operator==(const propagate_const_unique_ptr& x,
                                         std::nullptr_t) noexcept {
            return !x;
        }

        friend constexpr bool
        operator==(std::nullptr_t,
                   const propagate_const_unique_ptr& x) noexcept {
            return !x;
        }

        friend constexpr bool operator!=(const propagate_const_unique_ptr& x,
                                         std::nullptr_t) noexcept {
            return static_cast<bool>(x);
        }

        friend constexpr bool
        operator!=(std::nullptr_t,
                   const propagate_const_unique_ptr& x) noexcept {
            return static_cast<bool>(x);
        }

    private:
        template <typename U, typename D>
        friend class propagate_const_unique_ptr;

        std::unique_ptr<T, Deleter> ptr_;
    };

    // Comparisons between propagate_const_unique_ptrs
    template <typename T1, typename D1, typename T2, typename D2>
    inline bool
    operator==(const propagate_const_unique_ptr<T1, D1>& x,
               const propagate_const_unique_ptr<T2, D2>& y) noexcept {
        return x.get() == y.get();
    }

    template <typename T1, typename D1, typename T2, typename D2>
    inline auto
    operator<=>(const propagate_const_unique_ptr<T1, D1>& x,
                const propagate_const_unique_ptr<T2, D2>& y) noexcept {
        return x.get() <=> y.get();
    }

    template <typename T, typename... Args>
        requires(!std::is_array_v<T>)
    inline propagate_const_unique_ptr<T>
    make_propagate_const_unique(Args&&... args) {
        return propagate_const_unique_ptr<T>(
            std::make_unique<T>(std::forward<Args>(args)...));
    }

    template <typename T, typename Deleter>
    inline void swap(propagate_const_unique_ptr<T, Deleter>& lhs,
                     propagate_const_unique_ptr<T, Deleter>& rhs) noexcept {
        lhs.swap(rhs);
    }
} // namespace arborist
