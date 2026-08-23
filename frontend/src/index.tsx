import React, { useState } from "react";
import ReactDOM from "react-dom/client";
import getAll from "./api/get";
import { ToDoItems } from "./interfaces/toDoItems";

const App = () => {
  const [data, setData] = useState<string | ToDoItems | null>(null);
  const [error, setError] = useState<string | null>(null);
  React.useEffect(() => {
    const fetchData = async () => {
      const response = await getAll();
      if (response.error) {
        setError(response.error);
      } else {
        setData(response.data);
      }
    };
    fetchData();
  }, []); // fires once the App component has been loaded
  return (
    <div>
      {error ? (
        <div style={{ color: "red" }}>Error: {error}</div>
      ) : data ? (
        <div>Data loaded: {JSON.stringify(data)}</div>
      ) : (
        <div>Loading...</div>
      )}
    </div>
  );
};
const root = ReactDOM.createRoot(document.getElementById("root"));
root.render(<App />);
