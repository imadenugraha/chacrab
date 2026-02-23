use crate::{
    cli::display::short_id,
    core::models::VaultItem,
};

pub fn print_list_table(items: &[VaultItem]) {
    println!("ID        TYPE       TITLE                UPDATED");
    println!("------------------------------------------------------");
    for item in items {
        let id = short_id(&item.id.to_string());
        let kind = match item.r#type {
            crate::core::models::VaultItemType::Password => "password",
            crate::core::models::VaultItemType::Note => "note",
        };
        let title = truncate(&item.title, 20);
        let updated = item.updated_at.format("%Y-%m-%d").to_string();
        println!("{id:<8}  {kind:<9}  {title:<20}  {updated}");
    }
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_owned();
    }
    value.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
}
