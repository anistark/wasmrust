use yew::prelude::*;
use serde::{Deserialize, Serialize};
use gloo_storage::{LocalStorage, Storage};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Todo {
    id: usize,
    text: String,
    completed: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct TodoApp {
    todos: Vec<Todo>,
    next_id: usize,
    input_value: String,
    filter: Filter,
}

#[derive(Clone, Debug, PartialEq)]
enum Filter {
    All,
    Active,
    Completed,
}

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    AddTodo,
    ToggleTodo(usize),
    DeleteTodo(usize),
    UpdateInput(String),
    SetFilter(Filter),
    ClearCompleted,
}

impl Component for TodoApp {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let todos = LocalStorage::get("todos").unwrap_or_default();
        Self {
            todos,
            next_id: 1,
            input_value: String::new(),
            filter: Filter::All,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddTodo => {
                if !self.input_value.trim().is_empty() {
                    let todo = Todo {
                        id: self.next_id,
                        text: self.input_value.clone(),
                        completed: false,
                    };
                    self.todos.push(todo);
                    self.next_id += 1;
                    self.input_value.clear();
                    self.save_todos();
                }
            }
            Msg::ToggleTodo(id) => {
                if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
                    todo.completed = !todo.completed;
                    self.save_todos();
                }
            }
            Msg::DeleteTodo(id) => {
                self.todos.retain(|t| t.id != id);
                self.save_todos();
            }
            Msg::UpdateInput(value) => {
                self.input_value = value;
            }
            Msg::SetFilter(filter) => {
                self.filter = filter;
            }
            Msg::ClearCompleted => {
                self.todos.retain(|t| !t.completed);
                self.save_todos();
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        
        let filtered_todos: Vec<&Todo> = self.todos.iter()
            .filter(|todo| match self.filter {
                Filter::All => true,
                Filter::Active => !todo.completed,
                Filter::Completed => todo.completed,
            })
            .collect();

        html! {
            <div class="todoapp">
                <header class="header">
                    <h1>{"todos"}</h1>
                    <input
                        class="new-todo"
                        placeholder="What needs to be done?"
                        value={self.input_value.clone()}
                        oninput={link.callback(|e: InputEvent| {
                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                            Msg::UpdateInput(input.value())
                        })}
                        onkeypress={link.callback(|e: KeyboardEvent| {
                            if e.key() == "Enter" {
                                Msg::AddTodo
                            } else {
                                Msg::UpdateInput(String::new()) // No-op
                            }
                        })}
                    />
                </header>
                
                <section class="main">
                    <ul class="todo-list">
                        {for filtered_todos.iter().map(|todo| self.view_todo(todo, link))}
                    </ul>
                </section>
                
                <footer class="footer">
                    <span class="todo-count">
                        {format!("{} items left", self.active_count())}
                    </span>
                    
                    <ul class="filters">
                        {self.view_filter_button(&Filter::All, "All", link)}
                        {self.view_filter_button(&Filter::Active, "Active", link)}
                        {self.view_filter_button(&Filter::Completed, "Completed", link)}
                    </ul>
                    
                    if self.completed_count() > 0 {
                        <button 
                            class="clear-completed"
                            onclick={link.callback(|_| Msg::ClearCompleted)}
                        >
                            {"Clear completed"}
                        </button>
                    }
                </footer>
            </div>
        }
    }
}

impl TodoApp {
    fn view_todo(&self, todo: &Todo, link: &Scope<Self>) -> Html {
        html! {
            <li class={if todo.completed { "completed" } else { "" }}>
                <div class="view">
                    <input
                        class="toggle"
                        type="checkbox"
                        checked={todo.completed}
                        onchange={link.callback(move |_| Msg::ToggleTodo(todo.id))}
                    />
                    <label>{&todo.text}</label>
                    <button 
                        class="destroy"
                        onclick={link.callback(move |_| Msg::DeleteTodo(todo.id))}
                    />
                </div>
            </li>
        }
    }

    fn view_filter_button(&self, filter: &Filter, text: &str, link: &Scope<Self>) -> Html {
        let selected = self.filter == *filter;
        let filter = filter.clone();
        
        html! {
            <li>
                <a
                    class={if selected { "selected" } else { "" }}
                    onclick={link.callback(move |_| Msg::SetFilter(filter.clone()))}
                >
                    {text}
                </a>
            </li>
        }
    }

    fn active_count(&self) -> usize {
        self.todos.iter().filter(|t| !t.completed).count()
    }

    fn completed_count(&self) -> usize {
        self.todos.iter().filter(|t| t.completed).count()
    }

    fn save_todos(&self) {
        LocalStorage::set("todos", &self.todos).unwrap_or_default();
    }
}

fn main() {
    yew::Renderer::<TodoApp>::new().render();
}
