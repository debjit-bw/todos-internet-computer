use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{
    query, update,
};
use std::cell::RefCell;
use std::collections::BTreeMap;

type TodoTreeStore = BTreeMap<Principal, TodoTree>;

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct PartialTodo {
    text: String,
    completed: bool,
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct Todo {
    id: usize,
    text: String,
    completed: bool,
}

#[derive(Clone, Debug, Default, CandidType, Deserialize)]
struct TodoTree {
    pub count: usize,
    pub todos: BTreeMap<usize, Todo>
}

thread_local! {
    static TODOTREE: RefCell<TodoTreeStore> = RefCell::default();
}

#[query(name = "getPaginatedTodos")]
fn get_paginated_todos(offset: usize, limit: usize) -> Vec<Todo> {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let todo_tree = &profile_store.borrow();
        let todo_tree = todo_tree.get(&id).unwrap();
        let mut todos = Vec::<Todo>::new();
        for (_, v) in todo_tree.todos.range(offset..) {
            todos.push(v.clone());
            if todos.len() == limit as usize {
                break;
            }
        }
        todos
    })
}

#[query(name = "getTodo")]
fn get_todo(id: usize) -> Todo {
    let principal_id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let todo_tree = profile_store.borrow();
        let todo_tree = todo_tree.get(&principal_id).unwrap();
        todo_tree.todos.get(&id).unwrap().clone()
    })
}

#[update]
fn add_todos(todos: Vec<String>) {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let mut todo_tree = profile_store
            .borrow_mut()
            .get(&id)
            .cloned().unwrap_or_default();
        for (i, text) in todos.iter().enumerate() {
            todo_tree.todos.insert(
                todo_tree.count + i as usize,
                Todo {
                    id: todo_tree.count + i as usize,
                    text: text.clone(),
                    completed: false,
                },
            );
        }
        todo_tree.count += todos.len() as usize;
        profile_store.borrow_mut().insert(id, todo_tree);
    });
}

#[update]
fn remove_todos(ids: Vec<usize>) {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let mut todo_tree = profile_store
            .borrow_mut()
            .get(&id)
            .cloned().unwrap_or_default();
        for id in ids {
            todo_tree.todos.remove(&id);
        }
        profile_store.borrow_mut().insert(id, todo_tree);
    });
}

#[update]
fn toggle_todo(todo_id: usize) {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let mut todo_tree = profile_store
            .borrow_mut()
            .get(&id)
            .cloned().unwrap_or_default();
        let todo = todo_tree.todos.get_mut(&todo_id).unwrap();
        todo.completed = !todo.completed;
        profile_store.borrow_mut().insert(id, todo_tree);
    });
}