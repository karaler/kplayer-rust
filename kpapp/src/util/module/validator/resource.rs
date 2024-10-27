use std::collections::HashSet;
use validator::ValidationError;
use crate::util::module::resource::KPAppResourceItem;

// Custom validation function for unique names in KPAppResourceItem
pub fn validate_unique_names(items: &Vec<KPAppResourceItem>) -> Result<(), ValidationError> {
    let mut seen = HashSet::new();
    for item in items {
        if !seen.insert(&item.name) {
            return Err(ValidationError::new("duplicate_name"));
        }
    }
    Ok(())
}