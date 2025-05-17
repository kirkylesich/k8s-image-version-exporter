use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ImageRef {
    pub repository: String,
    pub tag: String,
}

impl ImageRef {
    pub fn parse(image: &str) -> Option<Self> {
        // image may be in form repo[:tag]
        let parts: Vec<&str> = image.split(':').collect();
        match parts.as_slice() {
            [repo, tag] => Some(ImageRef { repository: repo.to_string(), tag: tag.to_string() }),
            [repo] => Some(ImageRef { repository: repo.to_string(), tag: "latest".into() }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_with_tag() {
        let img = ImageRef::parse("nginx:1.0").unwrap();
        assert_eq!(img.repository, "nginx");
        assert_eq!(img.tag, "1.0");
    }

    #[test]
    fn parse_without_tag() {
        let img = ImageRef::parse("nginx").unwrap();
        assert_eq!(img.repository, "nginx");
        assert_eq!(img.tag, "latest");
    }
}
