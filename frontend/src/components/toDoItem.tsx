import React, { useEffect, useState } from "react";
import { updateToDoItemCall } from "../api/update";
import { deleteToDoItemCall } from "../api/delete";
import { TaskStatus } from "../interfaces/toDoItems";

interface ToDoItemProps {
  title: string;
  status: string;
  id: number;
  passBackResponse: (response: any) => void;
}

export const ToDoItem: React.FC<ToDoItemProps> = ({
  title,
  status,
  id,
  passBackResponse,
}) => {
  console.log(title, status, id);
  const [itemTitle, setTitle] = useState<string>(title);
  const [button, setButton] = useState<string>("");
  
  useEffect(() => {
    const processStatus = (status: string): string => {
      return status === "PENDING" ? "done" : "delete";
    };
    setButton(processStatus(status));
  }, [status]);
  
  const doneOrDelete = async () => {
    if (button === "done") {
      await updateToDoItemCall(itemTitle, TaskStatus.DONE, id).then((response) => {
        passBackResponse(response);
      });
    } else {
      await deleteToDoItemCall(itemTitle).then((response) => {
        passBackResponse(response);
      });
    }
  };
  
  return (
    <div className="itemContainer" id={`${id}`}>
      <p>{itemTitle}</p>
      <button className="actionButton" onClick={doneOrDelete}>
        {button}
      </button>
    </div>
  );
};
