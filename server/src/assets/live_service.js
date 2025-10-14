document.addEventListener("DOMContentLoaded", async function () {
  let response = await fetch("./book.html");
  let book_html = await response.text();
  let child_document = Document.parseHTMLUnsafe(book_html);
  console.log(child_document);
  document.body.appendChild(child_document.body);
  const evtSource = new EventSource("events", {});
  evtSource.onmessage = (event) => {
    console.log(event);
  };
});