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
