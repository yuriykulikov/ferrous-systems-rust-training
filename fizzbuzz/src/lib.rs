/// Iterates through a range mapping numbers using [to_fizzbuzz_string]
pub fn fizzbuzz_with_match(count: usize) -> Vec<String> {
    (1..)
        .into_iter()
        .map(|i| to_fizzbuzz_string(i as i32))
        .take(count)
        .collect()
}

fn to_fizzbuzz_string(i: i32) -> String {
    match (i % 3, i % 5) {
        (0, 0) => "FizzBuzz".to_owned(),
        (0, _) => "Fizz".to_owned(),
        (_, 0) => "Buzz".to_owned(),
        _ => i.to_string(),
    }
}

/// Creates a vector containing range values 1..=count.
/// Mutates the vector by changing each 3rd, 5th and 15th element
pub fn fizzbuzz_with_loops(count: usize) -> Vec<String> {
    let mut fb: Vec<String> = (1..=count).map(|i| i.to_string()).collect();
    for i in (0..=count).step_by(3).skip(1) {
        fb[i - 1] = "Fizz".to_owned()
    }
    for i in (0..=count).step_by(5).skip(1) {
        fb[i - 1] = "Buzz".to_owned()
    }
    for i in (0..=count).step_by(15).skip(1) {
        fb[i - 1] = "FizzBuzz".to_owned()
    }
    fb
}

#[cfg(test)]
mod test {
    use crate::{fizzbuzz_with_loops, fizzbuzz_with_match};

    const EXPECTED: [&str; 100] = [
        "1", "2", "Fizz", "4", "Buzz", "Fizz", "7", "8", "Fizz", "Buzz", "11", "Fizz", "13", "14",
        "FizzBuzz", "16", "17", "Fizz", "19", "Buzz", "Fizz", "22", "23", "Fizz", "Buzz", "26",
        "Fizz", "28", "29", "FizzBuzz", "31", "32", "Fizz", "34", "Buzz", "Fizz", "37", "38",
        "Fizz", "Buzz", "41", "Fizz", "43", "44", "FizzBuzz", "46", "47", "Fizz", "49", "Buzz",
        "Fizz", "52", "53", "Fizz", "Buzz", "56", "Fizz", "58", "59", "FizzBuzz", "61", "62",
        "Fizz", "64", "Buzz", "Fizz", "67", "68", "Fizz", "Buzz", "71", "Fizz", "73", "74",
        "FizzBuzz", "76", "77", "Fizz", "79", "Buzz", "Fizz", "82", "83", "Fizz", "Buzz", "86",
        "Fizz", "88", "89", "FizzBuzz", "91", "92", "Fizz", "94", "Buzz", "Fizz", "97", "98",
        "Fizz", "Buzz",
    ];

    #[test]
    fn fizzbuzz_with_match_test() {
        let x: Vec<String> = fizzbuzz_with_match(100);
        assert_eq!(x, EXPECTED.to_vec());
    }

    #[test]
    fn fizzbuzz_with_loops_test() {
        let x: Vec<String> = fizzbuzz_with_loops(100);
        assert_eq!(x, EXPECTED.to_vec());
        println!("{:?}", x)
    }

    #[test]
    fn fizzbuzz_crosscheck_test() {
        assert_eq!(fizzbuzz_with_loops(10000), fizzbuzz_with_match(10000),);
    }
}
