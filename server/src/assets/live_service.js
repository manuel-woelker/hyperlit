document.addEventListener("DOMContentLoaded", async function () {
  let response = await fetch("./book.html");
  let book_html = await response.text();
  let child_document = Document.parseHTMLUnsafe(book_html);
  console.log(child_document);
  let document_body = child_document.body;
  let title_node = document_body.querySelector("h1");
  if (title_node) {
    let title_text = title_node.textContent;
    document.title = title_text;
  }
  document.body.appendChild(document_body);
  const evtSource = new EventSource("events2", {});
  evtSource.onmessage = (event) => {
    console.log(event);
  };
});