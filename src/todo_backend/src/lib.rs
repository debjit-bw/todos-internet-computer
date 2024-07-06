use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{
    query, update,
};
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
    pub order: BTreeSet<Todo>, // Using BTreeSet to maintain ordered todos
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
                .cloned()
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
fn add_todos(todos: Vec<String>) {
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
            todo_tree.todos.insert(todo_id, todo.clone());
            todo_tree.order.insert(todo);
            todo_tree.count += 1;
        }
    });
}

#[update(name = "removeTodos")]
fn remove_todos(ids: Vec<usize>) {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let mut profile_store = profile_store.borrow_mut();
        if let Some(todo_tree) = profile_store.get_mut(&id) {
            for id in ids {
                if let Some(todo) = todo_tree.todos.remove(&id) {
                    todo_tree.order.remove(&todo);
                }
            }
        }
    });
}

#[update]
fn toggle_todo(todo_id: usize) -> Result<(), String> {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let mut profile_store = profile_store.borrow_mut();
        if let Some(todo_tree) = profile_store.get_mut(&id) {
            if let Some(mut todo) = todo_tree.todos.remove(&todo_id) {
                todo.completed = !todo.completed;
                todo_tree.order.remove(&todo);
                todo_tree.order.insert(todo.clone());
                todo_tree.todos.insert(todo_id, todo);
                Ok(())
            } else {
                Err(format!("Todo with id {} not found", todo_id))
            }
        } else {
            Err("Todo list for this user not found".to_string())
        }
    })
}

#[update(name = "updateTodoText")]
fn update_todo_text(todo_id: usize, new_text: String) -> Result<(), String> {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let mut profile_store = profile_store.borrow_mut();
        if let Some(todo_tree) = profile_store.get_mut(&id) {
            if let Some(mut todo) = todo_tree.todos.remove(&todo_id) {
                todo.text = new_text;
                todo_tree.order.remove(&todo); // Remove old entry
                todo_tree.order.insert(todo.clone()); // Insert updated entry
                todo_tree.todos.insert(todo_id, todo);
                Ok(())
            } else {
                Err(format!("Todo with id {} not found", todo_id))
            }
        } else {
            Err("Todo list for this user not found".to_string())
        }
    })
}
