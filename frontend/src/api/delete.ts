import { ToDoItems } from "../interfaces/toDoItems";
import { deleteCall } from "./utils";
import { Url } from "./url";

export async function deleteToDoItemCall(name: string) {
  console.log("deleteToDoItemCall called with:", name);
  return deleteCall<ToDoItems>(new Url().deleteUrl(name), 200);
}
