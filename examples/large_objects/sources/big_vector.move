module example::big_vector {
  use moveos_std::object;

  struct VecU8 has key, store {
    values: vector<u8>
  }

  fun init() {
    let values = std::vector::empty<u8>();
    let b = 100000;
    while (b > 0){
        std::vector::push_back(&mut values, 255);
        b = b-1
    };
    let obj = object::new_named_object( VecU8 { values });
    object::transfer(obj, @example);
  }

  entry fun transfer(to: address) {
    let id = object::named_object_id<VecU8>();
    let obj = object::take_object_extend<VecU8>(id);
    object::transfer_extend(obj, to);
  }

}
