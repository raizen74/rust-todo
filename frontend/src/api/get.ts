import { ToDoItems } from "../interfaces/toDoItems";
import { getCall } from "./utils";
import { Url } from "./url";

// Wraps the getCall function to get all ToDoItems from the backend.
// This is a convenience function that can be used in the frontend to get all ToDoItems
// without having to specify the URL and expected response code each time.
export default async function getAll() {
  let response = await getCall<ToDoItems>(new Url().getAll, 200);
  return response;
}
