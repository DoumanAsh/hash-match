use hash_match::{Function, Matcher};

#[test]
fn perform_simple_matching() {
    fn branch_1(_: ()) -> &'static str {
        "branch_1"
    }
    fn branch_2(_: ()) -> &'static str {
        "branch_2"
    }
    fn branch_3(_: ()) -> &'static str {
        "branch_3"
    }
    fn branch_4(_: ()) -> &'static str {
        "branch_4"
    }
    fn branch_5(_: ()) -> &'static str {
        "branch_5"
    }

    fn default_branch(_: ()) -> &'static str {
        "default"
    }

    const MATCHER: Matcher<5, (), &'static str> = Matcher::new([
        (b"branch_1", Function(branch_1)),
        (b"branch_2", Function(branch_2)),
        (b"branch_3", Function(branch_3)),
        (b"branch_4", Function(branch_4)),
        (b"branch_5", Function(branch_5)),
    ], Function(default_branch));

    assert_eq!(MATCHER.call(b"invalid", ()), "default");
    assert_eq!(MATCHER.call(b"branch_1", ()), "branch_1");
    assert_eq!(MATCHER.call(b"branch_12", ()), "default");
    assert_eq!(MATCHER.call(b"branch_2", ()), "branch_2");
    assert_eq!(MATCHER.call(b"branch_3", ()), "branch_3");
    assert_eq!(MATCHER.call(b"branch_4", ()), "branch_4");
    assert_eq!(MATCHER.call(b"branch_5", ()), "branch_5");
}

#[test]
#[should_panic]
fn should_fail_on_collision() {
    fn default_branch(_: ()) {
    }

    let _matcher = Matcher::new([
        (b"test", Function(default_branch)),
        (b"test", Function(default_branch)),
    ], Function(default_branch));
}
