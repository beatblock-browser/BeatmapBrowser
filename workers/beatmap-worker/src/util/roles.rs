use uuid::Uuid;

pub fn is_admin(user_id: &Uuid) -> bool {
    // Single administrator as requested
    // cbaf3066-fd5a-5d47-8718-d355694b211f
    *user_id == Uuid::parse_str("cbaf3066-fd5a-5d47-8718-d355694b211f").unwrap()
}
