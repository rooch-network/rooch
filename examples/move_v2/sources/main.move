// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::move_v2 {
    /********************* enum type ***********************/
    use std::vector;

    #[data_struct]
    struct Value has copy,drop,store {
        value: u64
    }

    fun extract_value(v: &Value): u64 {
        v.value
    }

    #[data_struct]
    enum RadiusValue has copy,drop,store {
        V1{value: u64},
        V2{value: u64},
    }

    public fun extract_radius_value(radius: &RadiusValue): u64 {
        match(radius) {
            RadiusValue::V1{value} => *value,
            RadiusValue::V2{value} => *value,
        }
    }

    #[data_struct]
    enum Shape has copy,drop,store,key {
        Circle{radius: RadiusValue},
        Rectangle{width: u64, height: Value}
    }

    public entry fun call_enum() {
        let v = 123;
        let radius_value = RadiusValue::V1{value: 123};
        let shape = Shape::Circle{radius: radius_value};
        match (&shape) {
            Circle{radius}           => v = extract_radius_value(radius) * extract_radius_value(radius),
            Rectangle{width, height} => v = *width * extract_value(height),
        };
        let _v1 = v;
    }

    fun area(self: &Shape): u64 {
        match (self) {
            Circle{radius}           => extract_radius_value(radius) * extract_radius_value(radius),
            Rectangle{width, height} => *width * extract_value(height),
        }
    }

    #[private_generics(T)]
    fun f1<T: drop>(arg: T): T {
        arg
    }

    fun f2() {
        let radius_value = RadiusValue::V1{value: 123};
        let shape = Shape::Circle{radius: radius_value};
        //f1<Shape::Circle>(v);
        f1<Shape>(shape);
    }

    /********************* self receiver ***********************/

    struct S {value: u32} has drop;

    fun foo(self: &S, x: u32): u32 { self.value + x }

    fun self_receiver(): u32 {
        let s = S {value: 123};
        s.foo(1)
    }

    /********************* index notation ***********************/

    fun index_notation() {
        let vec = vector::empty<u32>();
        vector::push_back(&mut vec, 1);
        vector::push_back(&mut vec, 2);
        vector::push_back(&mut vec, 3);
        let v1 = &mut vec[0];
        *v1 += 1;
        assert!(vec[0] == 2);

        let v1 = &vec[1];
        assert!(*v1 == 1);
    }

    /********************* positional struct ***********************/

    struct Wrapped(u64) has copy,drop;
    fun positional_struct() {
        let w = Wrapped(123);
        let Wrapped(v) = w;
        assert!(v == 123);
    }

    /********************* partial parttern ***********************/
    struct Foo{ x: u8, y: u16, z: u32 }
    fun partial_pattern() {
        let f = Foo{ x: 1, y: 2, z: 3 };
        let Foo{ x, .. } = f;
        assert!(x == 1);
    }

}
