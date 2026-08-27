use glue::errors::NanoServiceError;
use glue::token::HeaderToken;
use to_do_dal::to_do_items::schema::AllToDoItems;
use to_do_dal::to_do_items::transactions::get::GetAll;

pub async fn get_all<T: GetAll>(token: &HeaderToken) -> Result<AllToDoItems, NanoServiceError> {
    let all_items = T::get_all().await?;
    Ok(AllToDoItems::from_vec(all_items))
}
