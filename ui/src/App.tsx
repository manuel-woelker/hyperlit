import './App.css'
import {Layout} from "./layout/Layout.tsx";


document.addEventListener("DOMContentLoaded", async () => {
  let response = await fetch("./api/structure.json");
  console.log(response);
  let book_html = await response.text();
  console.log(book_html);
});

function App() {

  return (
      <>
        <Layout/>
      </>
  )
}

export default App
