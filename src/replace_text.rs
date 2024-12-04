use regex::Regex;

fn convert_money_to_words(text: &str) -> String {
    let re = Regex::new(r"\$1$").unwrap();
    let search_text = re.replace_all(text, "one dollar").to_string();
    println!("1: {}", search_text);

    let re = Regex::new(r"\$([1-9][\.]*[0-9]*\s(million|billion|trillion))").unwrap();
    let search_text = re.replace_all(&search_text, "$1 dollars").to_string();
    println!("2: {}", search_text);

    let re = Regex::new(r"\$(?<m>[,0-9]+)").unwrap();
    re.replace_all(&search_text, "$m dollars").to_string()
}

#[test]
fn test_convert_money_to_words() {
    // Special Case for a singular
    let text = "$1";
    assert_eq!("one dollar".to_string(), convert_money_to_words(text));

    let text = "$1.25 million";
    assert_eq!(
        "1.25 million dollars".to_string(),
        convert_money_to_words(text)
    );

    let text = "$1 million";
    assert_eq!(
        "1 million dollars".to_string(),
        convert_money_to_words(text)
    );

    let text = "$1 billion";
    assert_eq!(
        "1 billion dollars".to_string(),
        convert_money_to_words(text)
    );

    let text = "$100 billion";
    assert_eq!(
        "100 billion dollars".to_string(),
        convert_money_to_words(text)
    );

    let text = "$1 trillion";
    assert_eq!(
        "1 trillion dollars".to_string(),
        convert_money_to_words(text)
    );

    let text = "$100";
    assert_eq!("100 dollars".to_string(), convert_money_to_words(text));

    let text = "$100,000";
    assert_eq!("100,000 dollars".to_string(), convert_money_to_words(text));

    let text = "$1,000,000";
    assert_eq!(
        "1,000,000 dollars".to_string(),
        convert_money_to_words(text)
    );
}
