const { invoke } = window.__TAURI__.tauri;

let current_event;

async function get_event_list() {
  const events = await invoke("get_event_list"); 
  let event_list = document.querySelector("#event-list");
  event_list.textContent = "";

  for(let event_id in events) {
    let li = document.createElement("li");
    li.textContent = event_id;
    li.classList = "event event-sidebar";
    li.setAttribute('data-id', event_id);
    for(let result in events[event_id]) {
      li.classList.add(result);
    }
    event_list.appendChild(li);
  }
}

async function generate_state() {
  const config = await invoke("get_config"); 
  let state_ul = document.querySelector("#state-base ul");
  
  // Create locales
  if(config.locales.length > 0){
    let li = document.createElement("li");
    li.innerHTML = '<label for="current_locale">Locale: <select name="current_locale" id="current_locale" data-method="set_locale" data-params-locale="++"></select></label>';
    let select = li.querySelector("select");
    for(let i in config.locales) {
      let option = document.createElement("option");
      if(config.default_locale == config.locales[i]){
        option.setAttribute("selected", "selected")
      }
      option.textContent = config.locales[i];
      select.appendChild(option);
    }
    state_ul.appendChild(li);
  }

  // Create Tiles
  if(config.tiles.length > 0) {
    let li = document.createElement("li");
    li.innerHTML = '<label for="current_tile">Tile: <select name="current_tile" id="current_tile" data-method="set_tile"></select></label>';
    let select = li.querySelector("select");
    for(let i in config.tiles) {
      let option = document.createElement("option");
      option.textContent = config.tiles[i];
      select.appendChild(option);
    }
    state_ul.appendChild(li);
  }

  // Add Reputations
  let reputation_ul = document.querySelector("#state-reputations ul");
  if(config.reputations.length > 0) {
    for(let i in config.reputations) {
      let name = config.reputations[i];
      let li = document.createElement("li");
      li.innerHTML = "<label for='reputation_"+name+"'>"+name+":<input id='reputation_"+name+"' type='number' value=0 data-method='set_reputation' /></label>";
      reputation_ul.appendChild(li);
    }
  }

  // Add Variables
  let variables_ul = document.querySelector("#state-variables ul");
  for(let name in config.variables) {
    let variable = config.variables[name];
    let li = document.createElement("li");
    switch (variable) {
      case "bool":
        li.innerHTML = "<label for='variable_"+name+"'>"+name+":<input id='variable_"+name+"' type='checkbox' data-method='set_variable_bool' data-id='"+name+"' data-kind='"+variable+"'/></label>"
        break;
      case "integer":
        li.innerHTML = "<label for='variable_"+name+"'>"+name+":<input id='variable_"+name+"' type='number' value=0 data-method='set_variable_int' data-id='"+name+"' data-kind='"+variable+"'/></label>"
        break;
      case "float":
        li.innerHTML = "<label for='variable_"+name+"'>"+name+":<input id='variable_"+name+"' type='number' step='0.1' value=0.0 data-method='set_variable_float' data-id='"+name+"' data-kind='"+variable+"'/></label>"
        break;
    }
    variables_ul.appendChild(li);
  }

  // Add Items
  const items = await invoke("get_items"); 
  let items_ul = document.querySelector("#state-items ul");

  for(let i in items) {
    const item = items[i];
    const name = item.name;
    const max = item.max_stack_count;
    let li = document.createElement("li");
    li.innerHTML = "<label for='variable_"+name+"'>"+name+":<input id='variable_"+name+"' type='number' min=0 max="+max+" data-method='set_item' data-id='"+item.id+"' value=0 /></label>"
    items_ul.appendChild(li);
  }
}

async function next_event() {
  current_event = await invoke("next_event"); 
  render_event(current_event);
}

function render_event(event) {
  let event_dom = document.querySelector("#current_event");
  event_dom.querySelector(".event-title").textContent = event.title;
  event_dom.querySelector(".event-description").textContent = event.description;
  let choices = event_dom.querySelector(".event-choices");
  choices.textContent = "";

  for(let choice in event.choices) {
    let c = event.choices[choice]
    let li = document.createElement("li");
    li.classList = "choice";
    li.setAttribute('data-id', c.id);
    li.textContent = c.text;
    choices.append(li);
  }

  hide("result");
  show("event");
}

function render_result(result) {
  let current_result = document.querySelector("#current_result");
  current_result.querySelector(".result-text").textContent = result.text;

  let mod_list = current_result.querySelector(".result-modifiers");
  if(result.modifiers) {
    for(let modifier in result.modifiers) {
      let mod = result.modifiers[modifier];
      let li = document.createElement("li");
      li.textContent = mod.kind + " " + mod.id + " " + mod.amount;
      mod_list.append(li);
    }
  }

  hide("event");
  show("result");
}

function show(element) {
  let current_element = document.querySelector("#current_" + element);
  if(current_element) {
    current_element.classList.remove("hidden");
  }
}
function hide(element) {
  let current_element = document.querySelector("#current_" + element);
  if(current_element) {
    current_element.classList.add("hidden");
  }
}

window.addEventListener("DOMContentLoaded", () => {
  document
    .querySelector("#next-event")
    .addEventListener("click", () => next_event());

  document
  .querySelector("#reload-db")
  .addEventListener("click", async () => {
    await invoke("reload_db")
    location = location;
  });

  window.addEventListener("load", () => {
    get_event_list();
    generate_state();
  });

  document.addEventListener("click", async (event) => {
    if (event.target.classList.contains("choice")){
      let id = parseInt(event.target.dataset.id);
      let result = await invoke("set_choice", {choiceId: id});
      render_result(result);
    };

    if (event.target.classList.contains("event-sidebar")){
      let id = event.target.dataset.id;
      let current_event = await invoke("load_event", { eventId: id});
      if(current_event) {
        render_event(current_event);
      }
    }
  });

  document.querySelector("#state").addEventListener("change", async (event) => {
    let target = event.target;
    let method = target.dataset.method;
    let params = {};

    switch (method) {
      case 'set_locale':
        params["locale"] = target.value;
        break;
      case 'set_tile':
        params["tile"] = target.value;
        break;
      case 'set_variable_bool':
        params["variable"] = target.dataset.id;
        params["value"] = target.value == "on" ? true : false;
        break;
      case 'set_variable_float':
        params["variable"] = target.dataset.id;
        params["value"] = parseFloat(target.value);
        break;
      case 'set_variable_int':
        params["variable"] = target.dataset.id;
        params["value"] = parseInt(target.value);
        break;
      case 'set_item':
        params["item"] = target.dataset.id;
        params["count"] = parseInt(target.value);
        break;
            
      default:
        break;
    }
    let result = await invoke(method, params);
    console.log(result);
  });
});
