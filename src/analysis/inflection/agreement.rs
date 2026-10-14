pub fn grammemes(number: i64) -> [&'static str; 2] {
    let last = number.rem_euclid(10);
    let last_two = number.rem_euclid(100);
    if last == 1 && last_two != 11 {
        ["sing", "nomn"]
    } else if (2..=4).contains(&last) && !(10..20).contains(&last_two) {
        ["sing", "gent"]
    } else {
        ["plur", "gent"]
    }
}
