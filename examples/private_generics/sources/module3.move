// module rooch_examples::module3 {
//     #[test_only]
//     use rooch_examples::module1::{new_box, get_box_value};
//
//     struct Data3 has copy, drop {
//         v: u64
//     }
//
//     #[test]
//     fun test3() {
//         let data3 = Data3 { v: 789 };
//         let box3 = new_box<Data3, Data3, u8>(data3);
//         assert!(get_box_value(&box3) == Data3 { v: 789 }, 3000);
//     }
// }