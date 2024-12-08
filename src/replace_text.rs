use regex::Regex;

pub fn convert_money_to_words(text: &str) -> String {
    let re = Regex::new(r"\$1$").unwrap();
    let search_text = re.replace_all(text, "one dollar").to_string();

    let re = Regex::new(r"\$([1-9][\.]*[0-9]*\s(million|billion|trillion))").unwrap();
    let search_text = re.replace_all(&search_text, "$1 dollars").to_string();

    let re = Regex::new(r"\$(?<m>[,0-9]+)").unwrap();
    re.replace_all(&search_text, "$m dollars").to_string()
}

pub fn clean_text(text: &str) -> String {
    let re = Regex::new(r"@BRK#").unwrap();
    let search_text = re.replace_all(text, ".").to_string();

    let re = Regex::new(r"^\n+").unwrap();
    let search_text = re.replace_all(&search_text, "").to_string();

    let re = Regex::new(r"\s*\n").unwrap();
    let search_text = re.replace_all(&search_text, "\n").to_string();

    let re = Regex::new(r"\n+").unwrap();
    re.replace_all(&search_text, "\n").to_string()
}

pub fn convert_speed_from_acronyms_to_full_text(text: &str) -> String {
    // KPH
    let re = Regex::new(r"kph").unwrap();
    let search_text = re.replace_all(text, "kilometers per hour").to_string();

    let re = Regex::new(r"k\.p\.h\.\n").unwrap();
    let search_text = re
        .replace_all(&search_text, "kilometers per hour.\n")
        .to_string();

    let re = Regex::new(r"k\.p\.h\.(\s+[A-Z])").unwrap();
    let search_text = re
        .replace_all(&search_text, "kilometers per hour.$1")
        .to_string();

    let re = Regex::new(r"k\.p\.h\.").unwrap();
    let search_text = re
        .replace_all(&search_text, "kilometers per hour")
        .to_string();

    // MPH
    let re = Regex::new(r"mph").unwrap();
    let search_text = re.replace_all(&search_text, "miles per hour").to_string();

    let re = Regex::new(r"m\.p\.h\.\n").unwrap();
    let search_text = re
        .replace_all(&search_text, "miles per hour.\n")
        .to_string();

    let re = Regex::new(r"m\.p\.h\.(\s+[A-Z])").unwrap();
    let search_text = re
        .replace_all(&search_text, "miles per hour.$1")
        .to_string();

    let re = Regex::new(r"m\.p\.h\.").unwrap();
    re.replace_all(&search_text, "miles per hour").to_string()
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

#[test]
fn test_convert_money_to_words_multiple_times() {
    let text = "from $1000 to $1,000,000";
    assert_eq!(
        "from 1000 dollars to 1,000,000 dollars".to_string(),
        convert_money_to_words(text)
    );
}

#[test]
fn should_strip_first_line_if_blank() {
    let text = "\n\ntest\n";
    assert_eq!(clean_text(text), "test\n".to_string());
}

#[test]
fn test_strip_blank_lines() {
    let text = "\ntest\n";
    assert_eq!(clean_text(text), "test\n".to_string());
}

#[test]
fn test_strip_whitespace_only_lines() {
    let text = "test\n \n\ntest\n";
    assert_eq!(clean_text(text), "test\ntest\n".to_string());
}

#[test]
fn should_strip_out_multiple_new_lines() {
    let text = "\n\n\n\n";
    assert_eq!(clean_text(text), "".to_string());
}

#[test]
fn test_convert_break_to_periods() {
    let text = "@BRK#";
    assert_eq!(clean_text(text), ".".to_string());
}

#[test]
fn test_strip_spaces_at_end_of_line() {
    let text = " \n";
    assert_eq!(clean_text(text), "\n".to_string());
}

#[test]
fn test_convert_speed_acronyms() {
    let text = "kph";
    assert_eq!(
        convert_speed_from_acronyms_to_full_text(text),
        "kilometers per hour".to_string()
    );

    let text = "k.p.h.";
    assert_eq!(
        convert_speed_from_acronyms_to_full_text(text),
        "kilometers per hour".to_string()
    );
    // Check for m.p.h. being at end of sentence, and keep period
    let text = "k.p.h. The";
    assert_eq!(
        convert_speed_from_acronyms_to_full_text(text),
        "kilometers per hour. The".to_string()
    );

    // Check for m.p.h. being at end of a paragraph, and keep period
    let text = "k.p.h.\n";
    assert_eq!(
        convert_speed_from_acronyms_to_full_text(text),
        "kilometers per hour.\n".to_string()
    );

    let text = "mph";
    assert_eq!(
        convert_speed_from_acronyms_to_full_text(text),
        "miles per hour".to_string()
    );

    let text = "m.p.h.";
    assert_eq!(
        convert_speed_from_acronyms_to_full_text(text),
        "miles per hour".to_string()
    );
    // Check for m.p.h. being at end of sentence, and keep period
    let text = "m.p.h. The";
    assert_eq!(
        convert_speed_from_acronyms_to_full_text(text),
        "miles per hour. The".to_string()
    );

    // Check for m.p.h. being at end of a paragraph, and keep period
    let text = "m.p.h.\n";
    assert_eq!(
        convert_speed_from_acronyms_to_full_text(text),
        "miles per hour.\n".to_string()
    );
}
