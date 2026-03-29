#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub entity_id: i64,
    pub name: String,
    pub r#ref: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct NewEntity {
    pub name: String,
    pub r#ref: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateEntity {
    pub name: Option<String>,
    pub r#ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityLookup {
    Id(i64),
    Name(String),
}

pub fn parse_lookup(input: &str) -> EntityLookup {
    if let Ok(id) = input.parse::<i64>() {
        EntityLookup::Id(id)
    } else {
        EntityLookup::Name(input.to_string())
    }
}

pub fn validate_name(name: &str) -> Result<(), &'static str> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("entity name cannot be empty");
    }
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err("entity name cannot be purely numeric");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numeric_lookup_as_id() {
        assert_eq!(parse_lookup("42"), EntityLookup::Id(42));
    }

    #[test]
    fn parses_non_numeric_lookup_as_name() {
        assert_eq!(parse_lookup("student"), EntityLookup::Name("student".into()));
    }

    #[test]
    fn rejects_empty_name() {
        assert!(validate_name("   ").is_err());
    }

    #[test]
    fn rejects_numeric_name() {
        assert!(validate_name("123").is_err());
    }

    #[test]
    fn accepts_dotted_name() {
        assert!(validate_name("student.permissions").is_ok());
    }
}
