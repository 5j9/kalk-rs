✨ **`README.md`**

# 🧮 kalk-rs: A Console RPN Calculator

`kalk-rs` is a simple, command-line Reverse Polish Notation (RPN) calculator implemented in Rust. It features basic arithmetic, constants, stack manipulation, memory storage, and unique support for non-ASCII numeric input.

## Features

  * **Reverse Polish Notation (RPN):** Calculates expressions using a stack.
  * **Basic Arithmetic:** Supports addition, subtraction, multiplication, division, and exponentiation.
  * **Constants:** `pi` and `e`.
  * **Memory Storage:** Store and recall values using custom string keys.
  * **Stack Management:** Swap (`<>`) and Clear (`c`) the stack.
  * **Last Answer:** Push the result of the last successful calculation onto the stack using `a`.
  * **Flexible Input:** Automatically converts Persian digits (e.g., `۱۲۳`) to standard digits and ignores commas (thousand separators) in number inputs.

## Prerequisites

You need to have **Rust** and **Cargo** installed.

## How to Run

1.  **Clone the repository (if applicable) or save the code to `main.rs`.**

2.  **Build and run the project:**

    ```bash
    cargo run
    ```

## 🚀 Usage Example

Start the calculator, then enter numbers and operators separated by spaces.

```
$ cargo run
Welcome to kalk-rs (RPN Calculator). Type 'exit' to quit.
Supported: +, -, *, /, **, sqrt, pi, e, <>, c, a, "key" sto, "key" rcl.
Stack: []
> 10 5 +
Stack: [15]
> 3 *
Stack: [45]
> "rate" sto    # Store 45.0 as "rate"
Stack: []
> 100 "rate" rcl /
Stack: [2.2222222222222223]
> 
```

## Supported Commands

| Command | Operation | Example | Stack Change |
| :--- | :--- | :--- | :--- |
| **+**, **-**, **\***, **/** | Binary Arithmetic | `5 3 +` | $(a, b) \to (a+b)$ |
| **\*\*** | Exponentiation | `2 10 **` | $(a, b) \to (a^b)$ |
| **sqrt** | Square Root | `9 sqrt` | $(a) \to (\sqrt{a})$ |
| **pi**, **e** | Push Constant | `pi` | $() \to (\pi)$ |
| **\<\>** | Swap last two items | `1 2 <>` | $(a, b) \to (b, a)$ |
| **c** | Clear the stack | `c` | $... \to ()$ |
| **a** | Recall Last Answer | `a` | $() \to (\text{last result})$ |
| **`"key"` sto** | Store value | `10 "rate" sto` | $(\text{val}, \text{key}) \to ()$ |
| **`"key"` rcl** | Recall value | `"rate" rcl` | $(\text{key}) \to (\text{val})$ |

