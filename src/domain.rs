use unicode_segmentation::UnicodeSegmentation;

pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

pub struct SubscriberName(String);
impl SubscriberName {
    pub fn parse(name: String) -> SubscriberName {
        let is_empty_or_whitespace = name.trim().is_empty();

        const NAME_MAX_LEN: usize = 256;
        let is_too_long = name.graphemes(true).count() > NAME_MAX_LEN;

        let forbidden_characters = ['/', '(', ')', '{', '}', '[', ']', '\\', '<', '>', '"'];
        let contains_forbidden_characters = name.chars().any(|c| forbidden_characters.contains(&c));

        let is_invalid_subscriber_name =
            is_empty_or_whitespace || is_too_long || contains_forbidden_characters;
        if is_invalid_subscriber_name {
            panic!("Subscriber name is invalid: [{}]", name)
        } else {
            Self(name)
        }
    }
}
impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
