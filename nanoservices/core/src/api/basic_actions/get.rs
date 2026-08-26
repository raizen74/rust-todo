use glue::errors::NanoServiceError;
use to_do_dal::to_do_items::schema::AllToDoItems;
use to_do_dal::to_do_items::transactions::get::GetAll;

pub async fn get_all<T: GetAll>() -> Result<AllToDoItems, NanoServiceError> {
    let all_items = T::get_all().await?;
    Ok(AllToDoItems::from_vec(all_items))
}
