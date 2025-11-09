async function reload_book() {
  let response = await fetch("./book.html");
  let book_html = await response.text();
  let child_document = Document.parseHTMLUnsafe(book_html);
  let document_body = child_document.body;
  let title_node = document_body.querySelector("h1");
  if (title_node) {
    let title_text = title_node.textContent;
    document.title = title_text;
  }
  document.body.innerHTML = "";
  document.body.appendChild(document_body);
  document.querySelectorAll("[sloc]").forEach(sloc_element => {
    sloc_element.contentEditable = "true";
  });

}

document.addEventListener("DOMContentLoaded", async function () {
  document.addEventListener("beforeinput", (event) => {
    console.log(event.target.attributes.sloc);
    console.log(event.target.innerText);
  });
  document.addEventListener("input", (event) => {
    console.log(event.target.innerText);
    console.log(window.getSelection());
  });
  reload_book();
  const evtSource = new EventSource("events", {});
  evtSource.onmessage = (event) => {
    console.log(event);
    reload_book();
  };
});