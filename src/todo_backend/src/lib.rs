use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{query, update};
use std::cell::RefCell;
use std::collections::{HashMap, BTreeSet};

type TodoTreeStore = HashMap<Principal, TodoTree>;

#[derive(Clone, Debug, Default, CandidType, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
struct Todo {
    id: usize,
    text: String,
    completed: bool,
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct TodoTree {
    pub count: usize,
    pub todos: HashMap<usize, Todo>,
    pub order: BTreeSet<usize>, // Store only IDs to maintain order
}

thread_local! {
    static TODOTREE: RefCell<TodoTreeStore> = RefCell::default();
}

#[query(name = "getPaginatedTodos")]
fn get_paginated_todos(offset: usize, limit: usize) -> Vec<Todo> {
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
fn get_todo(id: usize) -> Option<Todo> {
    let principal_id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let todo_tree = profile_store.borrow();
        todo_tree.get(&principal_id).and_then(|tree| tree.todos.get(&id).cloned())
    })
}

#[update(name = "addTodos")]
fn add_todos(todos: Vec<String>) -> usize {
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
fn remove_todos(ids: Vec<usize>) -> usize {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let length = ids.len();
        let mut profile_store = profile_store.borrow_mut();
        if let Some(todo_tree) = profile_store.get_mut(&id) {
            for id in ids {
                todo_tree.todos.remove(&id);
                todo_tree.order.remove(&id);
            }
            todo_tree.count -= length;
            todo_tree.count
        } else {
            0
        }
    })
}

#[update(name = "toggleTodo")]
fn toggle_todo(todo_id: usize) -> Result<(bool), String> {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let mut profile_store = profile_store.borrow_mut();
        if let Some(todo_tree) = profile_store.get_mut(&id) {
            if let Some(todo) = todo_tree.todos.get_mut(&todo_id) {
                todo.completed = !todo.completed;
                Ok(todo.completed)
            } else {
                Err(format!("Todo with id {} not found", todo_id))
            }
        } else {
            Err("Todo list for this user not found".to_string())
        }
    })
}

#[update(name = "updateTodoText")]
fn update_todo_text(todo_id: usize, new_text: String) -> Result<(Todo), String> {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let mut profile_store = profile_store.borrow_mut();
        if let Some(todo_tree) = profile_store.get_mut(&id) {
            if let Some(todo) = todo_tree.todos.get_mut(&todo_id) {
                todo.text = new_text;
                Ok(todo.clone())
            } else {
                Err(format!("Todo with id {} not found", todo_id))
            }
        } else {
            Err("Todo list for this user not found".to_string())
        }
    })
}
