use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

#[derive(Debug)]
pub struct SubscriberName(String);
impl SubscriberName {
    pub fn parse(name: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = name.trim().is_empty();

        const NAME_MAX_LEN: usize = 256;
        let is_too_long = name.graphemes(true).count() > NAME_MAX_LEN;

        let forbidden_characters = ['/', '(', ')', '{', '}', '[', ']', '\\', '<', '>', '"'];
        let contains_forbidden_characters = name.chars().any(|c| forbidden_characters.contains(&c));

        let is_invalid_subscriber_name =
            is_empty_or_whitespace || is_too_long || contains_forbidden_characters;
        if is_invalid_subscriber_name {
            Err(format!(
                "Subscriber name contains invalid characters: [{}]",
                name
            ))
        } else {
            Ok(Self(name))
        }
    }
}
impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_graphene_long_name_is_valid() {
        let name = "ё".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphenes_is_rejected() {
        let name = "ё".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn a_whitespace_only_name_is_rejected() {
        let name = " ".repeat(250);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn a_empty_name_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Hariharan Narayanan".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
