pub fn dir_to_title(dir_name: &str) -> String {
    let mut title = String::with_capacity(dir_name.len() + 10);
    let mut capitalize_next = true;

    for ch in dir_name.chars() {
        match ch {
            '-' | '_' => {
                title.push(' ');
                capitalize_next = true;
            }
            c if capitalize_next => {
                title.extend(c.to_uppercase());
                capitalize_next = false;
            }
            c => title.push(c),
        }
    }
    title
}
