use regex::Regex;

fn process_line(text: &str) -> (String, String) {
    if let None = text.chars().next() {
        return ("".to_string(), "".to_string());
    }
    if text.chars().next().unwrap() == '#' {
        return ("".to_string(), "".to_string());
    }
    let ret = text.split_once("==");
    match ret {
        Some((x, y)) => return (x.to_string(), y.to_string()),
        None => {
            println!("Ignoring line, no '==' found: {}", text);
            ("".to_string(), "".to_string())
        }
    }
    //assert_eq!("cfg=".split_once('='), Some(("cfg", "")));
}

fn process_file_text(text: &str) -> Vec<(String, String)> {
    let mut ret: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        ret.push(process_line(line));
    }
    ret
}

fn process_user_replacements(text: &str, replacements: Vec<(String, String)>) -> String {
    let mut ret = text.to_string();

    for replace in replacements {
        let re = Regex::new(&replace.0).unwrap();
        ret = re.replace_all(&ret, &replace.1).to_string();
    }
    ret.to_string()
}

#[test]
fn test_split_valid_replacement() {
    let text = "word==WORD";
    assert_eq!(process_line(text), ("word".to_string(), "WORD".to_string()));
}

#[test]
fn test_return_empty_for_empty_string() {
    let text = "";
    assert_eq!(process_line(text), ("".to_string(), "".to_string()));
}

#[test]
fn test_return_empty_for_missing_splitter() {
    let text = "word  WORD";
    assert_eq!(process_line(text), ("".to_string(), "".to_string()));
}

#[test]
fn test_process_line_is_comment() {
    let text = "# comment";
    assert_eq!(process_line(text), ("".to_string(), "".to_string()));

    let text = "      # comment";
    assert_eq!(process_line(text), ("".to_string(), "".to_string()));

    let text = "# WORD==word";
    assert_eq!(process_line(text), ("".to_string(), "".to_string()));
}

// Currently not going to support this
/*#[test]
fn test_process_line_has_comment_at_end() {
    let text = "WORD==word # Comment at end";
    assert_eq!(process_line(text), ("WORD".to_string(), "word".to_string()));
}*/

#[test]
fn should_return_vector_of_replacements() {
    let text = "word==WORD\n\
            hi==hello";
    let results = vec![
        ("word".to_string(), "WORD".to_string()),
        ("hi".to_string(), "hello".to_string()),
    ];

    assert_eq!(process_file_text(text), results);
}

#[test]
fn should_apply_all_replacements() {
    let replacements = process_file_text(
        "word==WORD\n\
            hi==hello",
    );

    let text = "hi there, word to your brother";
    let expected = "hello there, WORD to your brother";
    let result = process_user_replacements(text, replacements);
    assert_eq!(expected, result);
}
