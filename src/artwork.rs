use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArtworkImageUrl {
    pub thumb_mini: String,
    pub small: String,
    pub regular: String,
    pub original: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArtworkImageNsfw {
    pub drawings: f64,
    pub hentai: f64,
    pub neutral: f64,
    pub porn: f64,
    pub sexy: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArtworkModerate {
    #[serde(rename = "type")]
    pub art_type: Option<String>,
    pub status: Option<String>,
    pub reason: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArtworkImage {
    pub urls: Option<ArtworkImageUrl>,
    pub nsfw: Option<ArtworkImageNsfw>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArtworkInfo {
    pub art_id: i64,
    pub title: String,
    pub tag_str: String,
    pub characters: Vec<String>,
    pub view_count: i32,
    pub like_count: i32,
    pub love_count: i32,
    pub artist_id: i64,
    pub upload_timestamp: i64,
    pub is_404: Option<bool>,
    pub sl: Option<i32>,
    pub images: Option<Vec<ArtworkImage>>,
    pub moderate: Option<ArtworkModerate>,
}

#[cfg(test)]
mod tests {
    use super::ArtworkImageUrl;
    use serde_json::json;

    #[test]
    fn test_json_marshal_basic() {
        assert_eq!(json!([1, 2, 3]).to_string(), "[1,2,3]");
    }

    #[test]
    fn test_json_marshal_custom_type() {
        assert_eq!(
            json!(ArtworkImageUrl {
                thumb_mini: "".to_string(),
                small: "".to_string(),
                regular: "".to_string(),
                original: "".to_string(),
            })
            .to_string(),
            "{\
                \"thumb_mini\":\"\",\
                \"small\":\"\",\
                \"regular\":\"\",\
                \"original\":\"\"\
            }",
        );
    }

    #[test]
    fn test_json_marshal_option_value_becomes_null() {
        let missing_original: Option<String> = None;
        assert_eq!(
            json!({
                "thumb_mini": "",
                "small": "",
                "regular": "",
                "original": missing_original,
            })
            .to_string(),
            "{\
                \"thumb_mini\":\"\",\
                \"small\":\"\",\
                \"regular\":\"\",\
                \"original\":null\
            }",
        );
    }

    #[test]
    fn test_json_marshal_vec() {
        assert_eq!(json!(vec!(1, 2, 3)).to_string(), "[1,2,3]");
    }
}
