fn main() {
    println!("Hello, World!");

    let _very_long_line_that_exceeds_the_default_maximum_length_of_one_hundred_characters_and_should_trigger_the_line_length_rule =
        42;

    let _trailing_whitespace = "test";

    // TODO: Fix this later
    let x = 5;

    println!("{}", x);
}
