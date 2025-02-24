// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::move_v2 {
    #[data_struct]
    enum Shape has copy,drop {
        Circle{radius: u64},
        Rectangle{width: u64, height: u64}
    }
    
    public entry fun call_enum() {
        let v = 123;
        let value = Shape::Circle{radius: 123};
        match (&value) {
            Circle{radius}           => v = *radius * *radius,
            Rectangle{width, height} => v = *width * *height
        };
        let _v1 = v;
    }

    fun area(self: &Shape): u64 {
        match (self) {
            Circle{radius}           => *radius * *radius,
            Rectangle{width, height} => *width * *height
        }
    }

    #[private_generics(T)]
    fun f1<T: drop>(arg: T): T {
        arg
    }

    fun f2() {
        let v = Shape::Circle{radius: 123};
        //f1<Shape::Circle>(v);
        f1<Shape>(v);
    }
}
