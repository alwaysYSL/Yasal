#[test]
fn test_escape_key_two_step() {
    let mut query = String::from("visual studio");
    let mut is_open = true;

    // Step 1: Query is not empty -> Esc clears query
    if !query.is_empty() {
        query.clear();
    } else {
        is_open = false;
    }
    assert!(query.is_empty());
    assert!(is_open);

    // Step 2: Query is empty -> Esc closes window
    if !query.is_empty() {
        query.clear();
    } else {
        is_open = false;
    }
    assert!(!is_open);
}
