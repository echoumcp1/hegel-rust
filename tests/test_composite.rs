use hegel::generators::integers;
use hegel::TestCase;

#[hegel::composite]
fn composite_integer_generator(tc: TestCase, n: i32) -> i32 {
    let x = tc.draw(integers::<i32>().min_value(0).max_value(100));
    x + n
}

#[hegel::test]
fn test_composite_generation(tc: TestCase) {
    let x = tc.draw(composite_integer_generator(1));
    assert!(x > 0);
}
