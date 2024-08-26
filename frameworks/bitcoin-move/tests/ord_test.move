#[test_only]
module bitcoin_move::ord_test {

    use bitcoin_move::ord;
    
    #[test]
    public fun test_charm(){
        let charms = 0u16;
        assert!(!ord::is_set_charm(charms, ord::charm_coin_flag()), 1);
        charms = ord::set_charm(charms, ord::charm_coin_flag());
        
        assert!(!ord::is_set_charm(charms, ord::charm_cursed_flag()), 2);
        charms = ord::set_charm(charms, ord::charm_cursed_flag());
        
        assert!(!ord::is_set_charm(charms, ord::charm_epic_flag()), 3);
        charms = ord::set_charm(charms, ord::charm_epic_flag());

        assert!(!ord::is_set_charm(charms, ord::charm_legendary_flag()), 4);
        charms = ord::set_charm(charms, ord::charm_legendary_flag());

        assert!(!ord::is_set_charm(charms, ord::charm_lost_flag()), 5);
        charms = ord::set_charm(charms, ord::charm_lost_flag());

        assert!(!ord::is_set_charm(charms, ord::charm_nineball_flag()), 6);
        charms = ord::set_charm(charms, ord::charm_nineball_flag());

        assert!(!ord::is_set_charm(charms, ord::charm_rare_flag()), 7);
        charms = ord::set_charm(charms, ord::charm_rare_flag());

        assert!(!ord::is_set_charm(charms, ord::charm_reinscription_flag()), 8);
        charms = ord::set_charm(charms, ord::charm_reinscription_flag());

        assert!(!ord::is_set_charm(charms, ord::charm_unbound_flag()), 9);
        charms = ord::set_charm(charms, ord::charm_unbound_flag());

        assert!(!ord::is_set_charm(charms, ord::charm_uncommon_flag()), 10);
        charms = ord::set_charm(charms, ord::charm_uncommon_flag());

        assert!(!ord::is_set_charm(charms, ord::charm_vindicated_flag()), 11);
        charms = ord::set_charm(charms, ord::charm_vindicated_flag());

        assert!(!ord::is_set_charm(charms, ord::charm_mythic_flag()), 12);
        charms = ord::set_charm(charms, ord::charm_mythic_flag());

        assert!(!ord::is_set_charm(charms, ord::charm_burned_flag()), 13);
        charms = ord::set_charm(charms, ord::charm_burned_flag());
        
        assert!(ord::is_set_charm(charms, ord::charm_coin_flag()), 14);
        assert!(ord::is_set_charm(charms, ord::charm_cursed_flag()), 15);
        assert!(ord::is_set_charm(charms, ord::charm_epic_flag()), 16);
        assert!(ord::is_set_charm(charms, ord::charm_legendary_flag()), 17);
        assert!(ord::is_set_charm(charms, ord::charm_lost_flag()), 18);
        assert!(ord::is_set_charm(charms, ord::charm_nineball_flag()), 19);
        assert!(ord::is_set_charm(charms, ord::charm_rare_flag()), 20);
        assert!(ord::is_set_charm(charms, ord::charm_reinscription_flag()), 21);
        assert!(ord::is_set_charm(charms, ord::charm_unbound_flag()), 22);
        assert!(ord::is_set_charm(charms, ord::charm_uncommon_flag()), 23);
        assert!(ord::is_set_charm(charms, ord::charm_vindicated_flag()), 24);
        assert!(ord::is_set_charm(charms, ord::charm_mythic_flag()), 25);
        assert!(ord::is_set_charm(charms, ord::charm_burned_flag()), 26);
    }
}