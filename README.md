## Todo App Backend

This implements a backend for a TODO app on the Internet Computer using Rust. It provides CRUD (Create, Read, Update, Delete) operations for managing TODO items. The backend is designed to be efficient and easy to use.

### Data Structures

#### `BTreeSet`

- **Usage**: The `BTreeSet` is used to store the order of TODO IDs (only IDs, not elements).
- **Advantages**:
  - **Ordered**: Maintains the order of elements, which is useful for efficiently retrieving paginated results.
  - **Logarithmic Time Complexity**: Provides `O(log n)` time complexity for insertions, deletions, and lookups, making it efficient for maintaining ordered sets of IDs.

#### `HashMap`

- **Usage**: The `HashMap` is used to store TODO items and the TODO tree.
- **Advantages**:
  - **Fast Lookups**: Provides `O(1)` average time complexity for lookups, insertions, and deletions.
  - **Efficient Storage**: Efficiently stores key-value pairs, where the keys are TODO IDs and the values are TODO items.

### Functions

#### `get_paginated_todos(offset: usize, limit: usize) -> Vec<Todo>`

- **Input**: 
  - `offset`: The starting index from which to retrieve TODO items.
  - `limit`: The maximum number of TODO items to retrieve.
- **Output**: A vector of TODO items starting from the specified offset up to the specified limit.

#### `get_todo(id: usize) -> Option<Todo>`

- **Input**: 
  - `id`: The ID of the TODO item to retrieve.
- **Output**: An optional TODO item. If the item exists, it is returned; otherwise, `None` is returned.

#### `add_todos(todos: Vec<String>) -> usize`

- **Input**: 
  - `todos`: A vector of TODO item texts to be added.
- **Output**: The updated count of TODO items after the new items are added.

#### `remove_todos(ids: Vec<usize>) -> usize`

- **Input**: 
  - `ids`: A vector of TODO item IDs to be removed.
- **Output**: The updated count of TODO items after the specified items are removed.

#### `toggle_todo(todo_id: usize) -> ToggleResult`

- **Input**: 
  - `todo_id`: The ID of the TODO item to be toggled (completed or not completed).
- **Output**: A `ToggleResult` struct containing:
  - `state`: The new completed state of the TODO item (true if completed, false otherwise).
  - `error`: An optional error message if the TODO item was not found.

#### `update_todo_text(todo_id: usize, new_text: String) -> UpdateResult`

- **Input**: 
  - `todo_id`: The ID of the TODO item to be updated.
  - `new_text`: The new text for the TODO item.
- **Output**: An `UpdateResult` struct containing:
  - `todo`: An optional updated TODO item. If the update is successful, the updated TODO item is returned; otherwise, `None` is returned.
  - `error`: An optional error message if the TODO item was not found.

### Conclusion

This backend for the TODO app leverages `BTreeSet` and `HashMap` to optimize for ordered retrieval and fast lookups, respectively. Each function is designed to handle specific CRUD operations.