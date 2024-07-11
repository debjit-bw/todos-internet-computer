use candid::{CandidType, Principal};
use ic_cdk::{query, update};
use std::cell::RefCell;
use std::collections::{HashMap, BTreeSet};
use core::ops::Bound::{Excluded, Included, Unbounded};

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
    pub next_id: usize
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

#[query(name = "getEffPaginatedTodos")]
async fn get_paginated_todos_efficient(last_id: usize, limit: usize) -> Vec<Todo> {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let todo_tree = profile_store.borrow();
        let mut limit_ = limit;
        if let Some(todo_tree) = todo_tree.get(&id) {
            let range_start = match last_id {
                0 => {
                    limit_ -= 1;
                    Included(0)
                },
                _ => Excluded(last_id),
            };
            let mut desired_range = todo_tree.order.range((range_start, Included(last_id + limit_)));
            let mut fetched_todos = desired_range.cloned().collect::<Vec<usize>>();
            let mut length = fetched_todos.len();

            let mut multiple = 2;

            // Handle cases where there are not enough todos to fulfill the limit
            while fetched_todos.len() < limit && length > 0 {
                let new_start = last_id + limit_;
                desired_range = todo_tree.order.range((Excluded(new_start), Included(new_start + limit_ * multiple)));
                let new_todos = desired_range.cloned().collect::<Vec<usize>>();
                length = new_todos.len();
                fetched_todos.extend(new_todos);
                multiple *= 2;
            }

            fetched_todos
                .into_iter()
                .take(limit)
                .filter_map(|id| todo_tree.todos.get(&id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    })
}

#[query(name = "getPaginatedTodosInterview")]
fn get_paginated_todos_interview(last_id: usize, limit: usize) -> Vec<Todo> {
    let id = ic_cdk::api::caller();
    TODOTREE.with(|profile_store| {
        let todo_tree = profile_store.borrow();
        if let Some(todo_tree) = todo_tree.get(&id) {
            let range_start = match last_id {
                0 => Included(0),
                _ => Excluded(last_id),
            };
            let mut desired_range = todo_tree.order.range((range_start, Unbounded)).into_iter();
            let mut fetched_todos = Vec::<usize>::new();
            while fetched_todos.len() < limit {
                if let Some(id) = desired_range.next() {
                    fetched_todos.push(*id);
                } else {
                    break;
                }
            }
            
            fetched_todos
            .into_iter()
            .take(limit)
            .filter_map(|id| todo_tree.todos.get(&id).cloned())
            .collect()
        } else {
            Vec::new()
        }
    })
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
    get_todo_sync(principal_id, id)
}

fn get_todo_sync(principal_id: Principal, id: usize) -> Option<Todo> {
    TODOTREE.with(|profile_store| {
        let todo_tree = profile_store.borrow();
        todo_tree.get(&principal_id).and_then(|tree| tree.todos.get(&id).cloned())
    })
}

#[update(name = "addTodos")]
async fn add_todos(todos: Vec<String>) -> usize {
    let id = ic_cdk::api::caller();
    add_todos_sync(id, todos)
}

fn add_todos_sync(id: Principal, todos: Vec<String>) -> usize {
    TODOTREE.with(|profile_store| {
        let mut profile_store = profile_store.borrow_mut();
        let todo_tree = profile_store.entry(id).or_insert_with(TodoTree::default);

        for text in todos {
            let todo_id = todo_tree.next_id;
            let todo = Todo {
                id: todo_id,
                text,
                completed: false,
            };
            todo_tree.todos.insert(todo_id, todo);
            todo_tree.order.insert(todo_id);
            todo_tree.count += 1;
            todo_tree.next_id += 1;
        }
        todo_tree.count
    })
}

#[update(name = "removeTodos")]
async fn remove_todos(ids: Vec<usize>) -> usize {
    let id = ic_cdk::api::caller();
    remove_todos_sync(id, ids)
}

fn remove_todos_sync(id: Principal, ids: Vec<usize>) -> usize {
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

#[test]
fn add() {
    let id = Principal::anonymous();

    for i in 0..10 {
        let _ = add_todos_sync(id, vec![format!("Todo {}", i)]);
    }

    for i in 7..10 {
        let _  = remove_todos_sync(id, vec![i]);
    }

    for i in 0..3 {
        let _ = add_todos_sync(id, vec![format!("Todo {}", 11+i)]);
    }

    let todo_9 = get_todo_sync(id, 9);
    assert!(todo_9.is_none());

    let todo_11 = get_todo_sync(id, 11).unwrap();
    assert!(todo_11.id == 11);
}