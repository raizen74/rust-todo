import { ToDoItem, ToDoItems, TaskStatus } from "../interfaces/toDoItems";
import { patchCall } from "./utils";
import { Url } from "./url";

export async function updateToDoItemCall(name: string, status: TaskStatus, id: number) {
  console.log("updateToDoItemCall called with:", name, status, id);
  const toDoItem: ToDoItem = {
    id,
    title: name,
    status,
  };
  return patchCall<ToDoItem, ToDoItems>(new Url().update, toDoItem, 200);
}
