const { invoke } = window.__TAURI__.tauri;
const { writeText} = window.__TAURI__.clipboard;
async function increment(name) {
  return invoke("advance", {
    name: name
  });
}
async function decrement(name) {
  return invoke("previous", {
    name: name
  });
}
class Bookmark {
    constructor(name, url, episode, has_new ) {
        this.name = name;
        this.url = url;
        this.episode = episode;
        this.has_new = has_new;
    }
  build_entry(row) {
    row.className = "bookmark_row";
    if (this.has_new) {
      row.id = "has_new";
    } else {
      row.id = "";
    }
    if (row.cells.length === 5) {
      row.deleteCell(0);
      row.deleteCell(0);
      row.deleteCell(0);
      row.deleteCell(0);
      row.deleteCell(0);
    }
    let incr = document.createElement("button");
    let decr = document.createElement("button");
    let remove = document.createElement("button");
    let copy = document.createElement("button");
    let c0 = row.insertCell(0);
    let c1 = row.insertCell(1);
    let c2 = row.insertCell(2);
    let c3 = row.insertCell(3);
    let c4 = row.insertCell(4);
    c1.innerText = this.name;
    c2.innerText = this.episode;
    c3.innerText = this.url.substring(12,70);
    decr.textContent = "-1";
    decr.addEventListener("click", () => {
      decrement(this.name).then(function(entry) {
        const obj = JSON.parse(entry);
        const bookmark = new Bookmark(obj.name, obj.url, obj.episode, obj.has_new);
        bookmark.build_entry(row);
              })
    });
    incr.textContent = "+1";
    incr.addEventListener("click", () => {
      increment(this.name).then(function(entry) {
        const obj = JSON.parse(entry);
        const bookmark = new Bookmark(obj.name, obj.url, obj.episode, obj.has_new);
        bookmark.build_entry(row);
      });
    });
    copy.textContent = "copy";
    copy.addEventListener("click", () => {
      writeText(this.url);
    });
    remove.textContent = "x"
    remove.addEventListener("click", () => {
      invoke("remove",
        {name: this.name});
      row.parentNode.removeChild(row);
    });
    c4.appendChild(remove);
    c0.appendChild(decr);
    c0.appendChild(incr);
    c0.appendChild(copy);
    
  }
};
async function create_entry() {
  let table= document.getElementById("bookmark-table");
  let name = document.getElementById("create-name");
  let url = document.getElementById("create-url");
  let episode= document.getElementById("create-episode");
  if (name.value === "") {
    return;
  }
  if (url.value === "") {
    return;
  }
  if (episode.value === "") {
    return;
  }
  const bookmark_unchecked = new Bookmark(name.value, url.value, parseInt(episode.value), false);
  invoke("insert",
    {entry: JSON.stringify(bookmark_unchecked)}).then((result) => {

  const obj = JSON.parse(result);
  const bookmark = new Bookmark(obj.name, obj.url, obj.episode, obj.has_new);
  let row = table.insertRow(-1);
  bookmark.build_entry(row);
    
  })
  .catch((error) => {
    alert(error);
  })
  
  
}
async function fetch_bookmarks() {
  let table= document.getElementById("bookmark-table");
  const strings = invoke("fetch_bookmarks");
  strings.then(function(strings) {
    
  var tableHeaderRowCount = 1;
  var rowCount = table.rows.length;
  for (var i = tableHeaderRowCount; i < rowCount; i++) {
    table.deleteRow(tableHeaderRowCount);
  }
  strings.forEach((item) => {
    const obj = JSON.parse(item);
    const bookmark = new Bookmark(obj.name,obj.url, obj.episode, obj.has_new);
    
    let row = table.insertRow(-1);
    bookmark.build_entry(row);
  });
  })
}
window.addEventListener("DOMContentLoaded", () => {
  fetch_bookmarks();

  document
    .querySelector("#refresh")
    .addEventListener("click", () => fetch_bookmarks());
  document
    .querySelector("#create-button")
    .addEventListener("click", () => create_entry());
    });



