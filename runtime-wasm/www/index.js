import * as cuentitos from "cuentitos-wasm";

let form = document.getElementById('database-form');
let file = document.getElementById('database-file');
let buffer = document.getElementById('buffer');
let next = document.getElementById('next');
let skip_button = document.getElementById('skip');

let runtime_id = -1;

let waiting_for_choice = false;

fetch('dance.db').then(file => {
  file.arrayBuffer().then(data => {
    runtime_id = cuentitos.load(new Uint8Array(data));
    progress_story()

  })
})

next.addEventListener("click", (event) => { 
  progress_story() 
});

skip_button.addEventListener("click", (event) => { 
  skip() 
});

window.addEventListener("keydown", (event) => {
  if(event.code == "Space" || event.code == "Enter")
    progress_story()

  if(event.code == "S" || event.code == "s")
    skip()
});

function progress_story() {
  if(runtime_id >= 0 && !waiting_for_choice)
  {
    let result = JSON.parse(cuentitos.progress_story(runtime_id))
    process_entry(result)    
  }
}

function skip() {
  if(runtime_id >= 0 && !waiting_for_choice)
  {
    let result = JSON.parse(cuentitos.skip(runtime_id))

    for(const block in result.blocks){
      const entry = result.blocks[block]
      if(entry.Text != undefined) {
        add_text(entry.Text.text)  
      }
    }
  
    process_choices(result.choices)
  }  
}

function process_entry(entry) {
  add_text(entry.text, "text", "animate__fadeIn")
  process_choices(entry.choices)
}

function add_text(text, className, animation) {
  let wrapper = document.createElement('p')  
  wrapper.classList.add(className)
  wrapper.classList.add('animate__animated')
  wrapper.classList.add(animation)
  wrapper.append(text)
  buffer.append(wrapper)
  scroll_to_bottom()
}

function process_choices(choices) {
  waiting_for_choice = choices.length > 0
  
  next.disabled = waiting_for_choice
  skip_button.disabled = waiting_for_choice  

  for(let choice = 0; choice < choices.length; choice++)
  {
    var button = document.createElement('div')
    button.classList.add('choice')
    button.classList.add('animate__animated')
    button.classList.add('animate__fadeIn')
    button.classList.add('animate__delay-1s')
    button.append(choice)
    button.append(" - ")
    button.append(choices[choice])
    button.addEventListener('click', (event) => {
      choose(choice, choices[choice])
    })
    buffer.append(button)
  }
  scroll_to_bottom()
}

function choose(choice, text) {
  let buttons = document.querySelectorAll('.choice')
  buttons.forEach(button => {
    button.remove();
  });
  add_text(text, "selected-choice", "animate__fadeIn")

  var result = JSON.parse(cuentitos.pick_choice(runtime_id, choice));

  process_entry(result) 
}

function scroll_to_bottom() {
  buffer.scrollTo(0, buffer.scrollHeight);
}