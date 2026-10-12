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
