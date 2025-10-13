document.addEventListener("DOMContentLoaded", function() {
    let child_document = Document.parseHTMLUnsafe("<h2>Foobar</h2>");
    console.log(child_document);
    document.body.appendChild(child_document.body);
    const evtSource = new EventSource("events", {
    });
    evtSource.onmessage = (event) => {
      console.log(event);
    };
});