import "./App.css";
import React, { useState } from "react";
import ReactDOM from "react-dom/client";
import getAll from "./api/get";
import { ToDoItems } from "./interfaces/toDoItems";
import { CreateToDoItem } from "./components/createItemForm";
import { ToDoItem } from "./components/toDoItem";
import { LoginForm } from "./components/loginForm";

const App = () => {
  const [data, setData] = useState<string | ToDoItems | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loggedin, setLoggedin] = useState<boolean>(
    localStorage.getItem("token") !== null,
  );
  function setToken(token: string) {
    localStorage.setItem("token", token);
    setLoggedin(true);
  }
  React.useEffect(() => {
    const fetchData = async () => {
      const response = await getAll();
      if (response.error) {
        setError(response.error);
      } else {
        setData(response.data);
      }
    };
    if (loggedin) {
      fetchData();
    }
  }, [loggedin]); // fires once the App component has been loaded

  function reRenderItems(response: any) {
    console.log("reRenderItems called with response:", response);
    if (response.error) {
      alert(JSON.stringify(response));
      return;
    } else if (response.data) {
      setData(response.data);
      setError(null);
    } else {
      setError("Unknown error");
    }
  }

  if (localStorage.getItem("token") === null) {
    return <LoginForm setToken={setToken} />;
  }

  if (error) {
    return <div style={{ color: "red" }}>Error: {error}</div>;
  } else if (!data || typeof data === "string") {
    return <div>Loading...</div>;
  } else {
    return (
      <div className="App">
        <div className="mainContainer">
          <div className="header">
            <p>complete tasks: {data.done.length}</p>
            <p>pending tasks: {data.pending.length}</p>
          </div>
          <h1>Pending Items</h1>
          <div>
            {data.pending.map((item, index) => (
              <>
                <ToDoItem
                  key={item.title + item.status}
                  title={item.title}
                  status={item.status}
                  id={item.id}
                  passBackResponse={reRenderItems}
                />
              </>
            ))}
          </div>
          <h1>Done Items</h1>
          <div>
            {data.done.map((item, index) => (
              <>
                <ToDoItem
                  key={item.title + item.status}
                  title={item.title}
                  status={item.status}
                  id={item.id}
                  passBackResponse={reRenderItems}
                />
              </>
            ))}
          </div>
          <CreateToDoItem passBackResponse={reRenderItems} />
        </div>
      </div>
    );
  }
};
const rootElement = document.getElementById("root");
if (!rootElement) {
  throw new Error("Root element was not found");
}

const root = ReactDOM.createRoot(rootElement);
root.render(<App />);
