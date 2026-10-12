//! The sandbox's whole product: one small function, so that a ticket of the
//! scenario has something real to change.

/// The greeting for `name`; an empty name greets the world.
pub fn greet(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        "Hello, world!".to_string()
    } else {
        format!("Hello, {name}!")
    }
}

#[cfg(test)]
mod tests {
    use super::greet;

    #[test]
    fn greets_a_name() {
        assert_eq!(greet("sandbox"), "Hello, sandbox!");
    }

    #[test]
    fn an_empty_name_greets_the_world() {
        assert_eq!(greet("  "), "Hello, world!");
    }
}

/// The farewell for `name`; a blank name says goodbye to the world.
pub fn farewell(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        "Goodbye, world!".to_string()
    } else {
        format!("Goodbye, {name}!")
    }
}

#[cfg(test)]
mod farewell_tests {
    use super::farewell;

    #[test]
    fn says_goodbye_to_a_name() {
        assert_eq!(farewell("sandbox"), "Goodbye, sandbox!");
    }

    #[test]
    fn a_blank_name_says_goodbye_to_the_world() {
        assert_eq!(farewell(" "), "Goodbye, world!");
    }
}
