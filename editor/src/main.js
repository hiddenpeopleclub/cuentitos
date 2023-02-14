const { invoke } = window.__TAURI__.tauri;

let greetInputEl;
let greetMsgEl;

let current_event;

function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsgEl.textContent = invoke("greet", { name: greetInputEl.value });
}

async function next_event() {
  current_event = await invoke("next_event"); 
  let event_dom = document.querySelector("#current_event");
  event_dom.querySelector(".event-title").textContent = current_event.title;
  event_dom.querySelector(".event-description").textContent = current_event.description;
  let choices = event_dom.querySelector(".event-choices");
  choices.textContent = "";

  for(let choice in current_event.choices) {
    let c = current_event.choices[choice]
    let li = document.createElement("li");
    li.classList = "choice";
    li.setAttribute('data-id', c.id);
    li.textContent = c.text;
    choices.append(li);
  }

  event_dom.classList.remove("hidden");

}

window.addEventListener("DOMContentLoaded", () => {
  // greetInputEl = document.querySelector("#greet-input");
  // greetMsgEl = document.querySelector("#greet-msg");
  // document
  //   .querySelector("#greet-button")
  //   .addEventListener("click", () => greet());

  document
    .querySelector("#next-event")
    .addEventListener("click", () => next_event());

  document.addEventListener("click", (event) => {
    if (event.target.classList.contains("choice")){
      let id = event.target.dataset.id;
      console.log("Clicked on choce:" + id);
    };
  })
});
