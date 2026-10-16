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

/// `n, n-1, …, 1`; empty for zero.
pub fn countdown(n: u32) -> Vec<u32> {
    (1..=n).rev().collect()
}

#[cfg(test)]
mod countdown_tests {
    use super::countdown;

    #[test]
    fn counts_down_to_one() {
        assert_eq!(countdown(3), vec![3, 2, 1]);
    }

    #[test]
    fn zero_counts_nothing() {
        assert!(countdown(0).is_empty());
    }
}

/// The thanks for `name`; a blank name thanks the world.
pub fn thank(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        "Thank you, world!".to_string()
    } else {
        format!("Thank you, {name}!")
    }
}

#[cfg(test)]
mod thank_tests {
    use super::thank;

    #[test]
    fn thanks_a_name() {
        assert_eq!(thank("sandbox"), "Thank you, sandbox!");
    }

    #[test]
    fn a_blank_name_thanks_the_world() {
        assert_eq!(thank(" "), "Thank you, world!");
    }
}

/// The wave for `name`; a blank name waves to the world.
pub fn wave(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        "Hi, world!".to_string()
    } else {
        format!("Hi, {name}!")
    }
}

#[cfg(test)]
mod wave_tests {
    use super::wave;

    #[test]
    fn waves_to_a_name() {
        assert_eq!(wave("sandbox"), "Hi, sandbox!");
    }

    #[test]
    fn a_blank_name_waves_to_the_world() {
        assert_eq!(wave(" "), "Hi, world!");
    }
}

/// The cheer for `name`; a blank name cheers the world.
pub fn cheer(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        "Cheers, world!".to_string()
    } else {
        format!("Cheers, {name}!")
    }
}

#[cfg(test)]
mod cheer_tests {
    use super::cheer;

    #[test]
    fn cheers_a_name() {
        assert_eq!(cheer("sandbox"), "Cheers, sandbox!");
    }

    #[test]
    fn a_blank_name_cheers_the_world() {
        assert_eq!(cheer(" "), "Cheers, world!");
    }
}

/// The welcome for `name`; a blank name welcomes the world.
pub fn welcome(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        "Welcome, world!".to_string()
    } else {
        format!("Welcome, {name}!")
    }
}

#[cfg(test)]
mod welcome_tests {
    use super::welcome;

    #[test]
    fn welcomes_a_name() {
        assert_eq!(welcome("sandbox"), "Welcome, sandbox!");
    }

    #[test]
    fn a_blank_name_welcomes_the_world() {
        assert_eq!(welcome(" "), "Welcome, world!");
    }
}
