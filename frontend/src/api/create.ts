import { NewToDoItem, ToDoItem, ToDoItems, TaskStatus } from "../interfaces/toDoItems";
import { postCall } from "./utils";
import { Url } from "./url";

export async function createToDoItemCall(title: string) {
  console.log("createToDoItemCall called with:", title);
  const toDoItem: NewToDoItem = {
    title: title,
    status: TaskStatus.PENDING,
  };
  // Wraps the postCall function to create a new ToDoItem in the backend.
  // The return value is a promise that resolves to the response from the backend, which is either a ToDoItems object or an error message.
  return postCall<NewToDoItem, ToDoItems>(new Url().create, toDoItem, 201);
}
