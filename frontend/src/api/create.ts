import { ToDoItem, ToDoItems, TaskStatus } from "../interfaces/toDoItems";
import { postCall } from "./utils";
import { Url } from "./url";

export async function createToDoItemCall(title: string) {
  const toDoItem: ToDoItem = {
    title: title,
    status: TaskStatus.PENDING,
  };
  // Wraps the postCall function to create a new ToDoItem in the backend.
  // The return value is a promise that resolves to the response from the backend, which is either a ToDoItems object or an error message.
  return postCall<ToDoItem, ToDoItems>(new Url().create, toDoItem, 201);
}
