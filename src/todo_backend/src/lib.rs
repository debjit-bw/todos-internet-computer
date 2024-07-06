use candid::{CandidType, Principal};
use ic_cdk::{query, update};
use std::cell::RefCell;
use std::collections::{HashMap, BTreeSet};

type TodoTreeStore = HashMap<Principal, TodoTree>;

#[derive(Clone, CandidType)]
struct Todo {
    id: usize,
    text: String,
    completed: bool,
}

#[derive(Default, CandidType)]
struct TodoTree {
    pub count: usize,
    pub todos: HashMap<usize, Todo>,
    pub order: BTreeSet<usize>, // Store only IDs to maintain order
}

#[derive(CandidType)]
struct ToggleResult {
    state: bool,
    error: Option<String>,
}

#[derive(CandidType)]
struct UpdateResult {
    todo: Option<Todo>,
    error: Option<String>,
}

thread_local! {
    static TODOTREE: RefCell<TodoTreeStore> = RefCell::default();
}

#[query(name = "getPaginatedTodos")]
async fn get_paginated_todos(offset: usize, limit: usize) -> Vec<Todo> {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let todo_tree = profile_store.borrow();
        if let Some(todo_tree) = todo_tree.get(&id) {
            todo_tree.order
                .iter()
                .skip(offset)
                .take(limit)
                .filter_map(|id| todo_tree.todos.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    })
}

#[query(name = "getTodo")]
async fn get_todo(id: usize) -> Option<Todo> {
    let principal_id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let todo_tree = profile_store.borrow();
        todo_tree.get(&principal_id).and_then(|tree| tree.todos.get(&id).cloned())
    })
}

#[update(name = "addTodos")]
async fn add_todos(todos: Vec<String>) -> usize {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let mut profile_store = profile_store.borrow_mut();
        let todo_tree = profile_store.entry(id).or_insert_with(TodoTree::default);

        for text in todos {
            let todo_id = todo_tree.count;
            let todo = Todo {
                id: todo_id,
                text,
                completed: false,
            };
            todo_tree.todos.insert(todo_id, todo);
            todo_tree.order.insert(todo_id);
            todo_tree.count += 1;
        }
        todo_tree.count
    })
}

#[update(name = "removeTodos")]
async fn remove_todos(ids: Vec<usize>) -> usize {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let mut profile_store = profile_store.borrow_mut();
        if let Some(todo_tree) = profile_store.get_mut(&id) {
            for id in ids {
                todo_tree.todos.remove(&id);
                if todo_tree.order.remove(&id) {
                    todo_tree.count -= 1;
                }
            }
            todo_tree.count
        } else {
            0
        }
    })
}

#[update(name = "toggleTodo")]
async fn toggle_todo(todo_id: usize) -> ToggleResult {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let mut profile_store = profile_store.borrow_mut();
        if let Some(todo_tree) = profile_store.get_mut(&id) {
            if let Some(todo) = todo_tree.todos.get_mut(&todo_id) {
                todo.completed = !todo.completed;
                ToggleResult { state: todo.completed, error: None }
            } else {
                ToggleResult { state: false, error: Some(format!("Todo with id {} not found", todo_id)) }
            }
        } else {
            ToggleResult { state: false, error: Some("Todo list for this user not found".to_string()) }
        }
    })
}

#[update(name = "updateTodoText")]
async fn update_todo_text(todo_id: usize, new_text: String) -> UpdateResult {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let mut profile_store = profile_store.borrow_mut();
        if let Some(todo_tree) = profile_store.get_mut(&id) {
            if let Some(todo) = todo_tree.todos.get_mut(&todo_id) {
                todo.text = new_text;
                UpdateResult { todo: Some(todo.clone()), error: None }
            } else {
                UpdateResult { todo: None, error: Some(format!("Todo with id {} not found", todo_id)) }
            }
        } else {
            UpdateResult { todo: None, error: Some("Todo list for this user not found".to_string()) }
        }
    })
}
