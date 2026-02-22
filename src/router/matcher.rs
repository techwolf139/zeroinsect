use once_cell::sync::Lazy;
use regex::Regex;

static TOPIC_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_+/\-#$]*$").unwrap());

pub struct TopicMatcher;

impl TopicMatcher {
    pub fn is_valid_topic(topic: &str) -> bool {
        if topic.is_empty() || topic.len() > 65535 {
            return false;
        }

        if topic.contains("//") || topic.starts_with('/') || topic.ends_with('/') {
            return false;
        }

        TOPIC_REGEX.is_match(topic)
    }

    pub fn validate_subscription(topics: &[String]) -> Vec<bool> {
        topics.iter().map(|t| Self::is_valid_topic(t)).collect()
    }

    pub fn extract_user_id_from_topic(topic: &str) -> Option<String> {
        if let Some(start) = topic.find("chat/u/") {
            let user_id = &topic[start + 7..];
            if !user_id.contains('/') {
                return Some(user_id.to_string());
            }
        }
        None
    }

    pub fn extract_group_id_from_topic(topic: &str) -> Option<String> {
        if let Some(start) = topic.find("chat/g/") {
            let group_id = &topic[start + 7..];
            if !group_id.contains('/') {
                return Some(group_id.to_string());
            }
        }
        None
    }
}
