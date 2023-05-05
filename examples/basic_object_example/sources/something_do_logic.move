module rooch_examples::something_do_logic {
    use rooch_examples::something;
    use moveos_std::object::Object;
    use rooch_examples::something::SomethingProperties;

    friend rooch_examples::something_aggregate;

    public(friend) fun do_something(obj: Object<SomethingProperties>): Object<SomethingProperties> {
        let i = something::i(&obj) + 1;
        something::set_i(&mut obj, i);
        let j = something::j(&obj) + 1;
        something::set_j(&mut obj, j);
        obj
    }
}
