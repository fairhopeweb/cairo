// TODO(orizi): Split tests into modules.

func run_tests() -> felt {
    let test_count = 0;
    bool_tests(test_count);
    felt_tests(test_count);
    uint128_tests(test_count);
    uint128_divmod_tests(test_count);
    array_tests(test_count);
    test_count
}

func bool_tests(ref test_count: felt) {
    assert_and_count(test_count, true);
    assert_and_count(test_count, !false);
    assert_and_count(test_count, true == true);
    assert_and_count(test_count, false == false);
    assert_and_count(test_count, !(true == false));
    assert_and_count(test_count, !(false == true));
    assert_and_count(test_count, !(false & false));
    assert_and_count(test_count, !(true & false));
    assert_and_count(test_count, !(false & true));
    assert_and_count(test_count, true & true);
    assert_and_count(test_count, !(false | false));
    assert_and_count(test_count, true | false);
    assert_and_count(test_count, false | true);
    assert_and_count(test_count, true | true);
}

func felt_tests(ref test_count: felt) {
    assert_and_count(test_count, 1 + 3 == 4);
    assert_and_count(test_count, 3 + 6 == 9);
    assert_and_count(test_count, 3 - 1 == 2);
    assert_and_count(test_count, 1231 - 231 == 1000);
    assert_and_count(test_count, 1 * 3 == 3);
    assert_and_count(test_count, 3 * 6 == 18);
    assert_and_count(test_count, 1 < 4);
    assert_and_count(test_count, 1 <= 4);
    assert_and_count(test_count, !(4 < 4));
    assert_and_count(test_count, 4 <= 4);
    assert_and_count(test_count, 5 > 2);
    assert_and_count(test_count, 5 >= 2);
    assert_and_count(test_count, !(3 > 3));
    assert_and_count(test_count, 3 >= 3);
}

func uint128_tests(ref test_count: felt) {
    let u1 = uint128_from_felt(1);
    let u2 = uint128_from_felt(2);
    let u3 = uint128_from_felt(3);
    let u4 = uint128_from_felt(4);
    let u5 = uint128_from_felt(5);
    let u6 = uint128_from_felt(6);
    let u9 = uint128_from_felt(9);
    let u231 = uint128_from_felt(231);
    let u1000 = uint128_from_felt(1000);
    let u1231 = uint128_from_felt(1231);
    assert_and_count(test_count, u1 + u3 == u4);
    assert_and_count(test_count, u3 + u6 == u9);
    assert_and_count(test_count, u3 - u1 == u2);
    assert_and_count(test_count, u1231 - u231 == u1000);
    assert_and_count(test_count, u1 < u4);
    assert_and_count(test_count, u1 <= u4);
    assert_and_count(test_count, !(u4 < u4));
    assert_and_count(test_count, u4 <= u4);
    assert_and_count(test_count, u5 > u2);
    assert_and_count(test_count, u5 >= u2);
    assert_and_count(test_count, !(u3 > u3));
    assert_and_count(test_count, u3 >= u3);
}

func array_tests(ref test_count: felt) {
    let arr = array_new::<felt>();
    array_append::<felt>(arr, 10);
    array_append::<felt>(arr, 11);
    array_append::<felt>(arr, 12);
        assert_and_count(test_count, match array_at::<felt>(arr, uint128_from_felt(0)) {
            Option::Some(x) => x == 10,
            Option::None(()) => false,
    });
        assert_and_count(test_count, match array_at::<felt>(arr, uint128_from_felt(1)) {
            Option::Some(x) => x == 11,
            Option::None(()) => false,
    });
        assert_and_count(test_count, match array_at::<felt>(arr, uint128_from_felt(2)) {
            Option::Some(x) => x == 12,
            Option::None(()) => false,
    });
        assert_and_count(test_count, match array_at::<felt>(arr, uint128_from_felt(5)) {
            Option::Some(x) => false,
            Option::None(()) => true,
    });
}

func nz_u128_from_felt(val: felt) -> NonZero::<uint128> {
    match uint128_jump_nz(uint128_from_felt(val)) {
        JumpNzResult::Zero(()) => {
            let data = array_new::<felt>();
            array_append::<felt>(data, 7);
            panic(data)
        },
        JumpNzResult::NonZero(x) => x,
    }
}

func uint128_divmod_tests(ref test_count: felt) {
    let (q, r) = uint128_divmod(uint128_from_felt(8), nz_u128_from_felt(2));
    assert_and_count(test_count, q == uint128_from_felt(4));
    assert_and_count(test_count, r == uint128_from_felt(0));
    let (q, r) = uint128_divmod(uint128_from_felt(7), nz_u128_from_felt(3));
    assert_and_count(test_count, q == uint128_from_felt(2));
    assert_and_count(test_count, r == uint128_from_felt(1));
}

func assert_and_count(ref test_count: felt, cond: bool) {
    assert(cond, test_count);
    test_count = test_count + 1;
}
