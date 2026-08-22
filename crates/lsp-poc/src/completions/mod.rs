pub fn completion_trigger_characters() -> Vec<String> {
    let mut chars = vec![String::from("\""), String::from(":"), String::from(" ")];

    chars.sort();
    chars
}
