# **🧮 kalk-rs: A Console RPN Calculator**

kalk-rs is a simple, command-line Reverse Polish Notation (RPN) calculator implemented in Rust. It features a wide range of functions, including constants, advanced math, stack manipulation, memory storage, and unique support for non-ASCII numeric input.

## **Features**

* **Reverse Polish Notation (RPN):** Calculates expressions using a stack.  
* **Comprehensive Math:** Supports basic arithmetic, exponentiation, logarithms, percent change, and Euclidean remainder.  
* **Trigonometry & Conversions:** Includes standard trig functions (sin, cos, tan, etc.) and unit conversions between **degrees and radians**.  
* **Combinatorics:** Calculate **Factorials** (\!), **Permutations** (P), and **Combinations** (C).  
* **Constants:** pi and e.  
* **Memory Storage:** Store and recall values using custom string keys (sto, rcl).  
* **Stack Management:** Swap (\<\>) and Clear (c) the stack.  
* **Last Answer:** Push the result of the last successful calculation onto the stack using a.  
* **Base Display:** View the integer component of a number in **hexadecimal, binary, or octal** (hex, bin, oct).  
* **Flexible Input:** Automatically converts Persian/Arabic digits (e.g., ۱۲۳) to standard digits and ignores commas (thousand separators) in number inputs.

## **Prerequisites**

You need to have **Rust** and **Cargo** installed.

## **How to Run**

1. **Clone the repository (if applicable) or save the code to main.rs.**  
2. **Build and run the project:**  
   cargo run

## **🚀 Usage Example**

Start the calculator, then enter numbers and operators separated by spaces. The calculator will display the stack state after each entry.

$ cargo run  
Welcome to kalk-rs (RPN Calculator). Type 'exit' to quit.  
Type 'help' for a list of all functions or '"func" help' for specific usage.  
Stack: \[\]  
\> 45 rad sin pi 2 /  
Stack: \[0.7071067811865476, 1.5707963267948966\] \# sin(45 deg) and pi/2  
\> 5 2 C  
Stack: \[0.7071067811865476, 10\] \# 5 choose 2 is 10  
\> "pi" help \# Get specific usage  
\[help: pi | Push the value of pi\]  
Stack: \[0.7071067811865476, 10\]  
\> 100 120 %%  
Stack: \[0.7071067811865476, 20\] \# Percent change from 100 to 120 is 20%  
\> 

## **Supported Commands**

| Group | Command | Operation | Stack Notation (Input → Output) |
| :---- | :---- | :---- | :---- |
| **Arithmetic** | **\+**, **\-**, **\***, **/** | Basic Arithmetic | $(a, b) \\to (a+b)$ |
|  | **\*\*** | Power / Exponentiation | $(a, b) \\to (a^b)$ |
|  | **%** | Euclidean Remainder | $(a, b) \\to (a \\bmod b)$ |
|  | **%%** | Percent Change | $(a, b) \\to (\\frac{b-a}{a} \\times 100)$ |
|  | **log** | Logarithm | $(a, b) \\to (\\log\_{b}(a))$ |
| **Trigonometry** | **sin**, **cos**, **tan** | Trig functions (expects radians) | $(a) \\to (\\sin(a))$ |
|  | **acos**, **asin**, **atan** | Inverse Trig (result in radians) | $(a) \\to (\\text{acos}(a))$ |
|  | **atan2** | Arc tangent of $y/x$ | $(y, x) \\to (\\text{atan2}(y, x))$ |
| **Unary** | **sqrt** | Square Root | $(a) \\to (\\sqrt{a})$ |
|  | **exp** | $e$ raised to the power of $a$ | $(a) \\to (e^a)$ |
|  | **ceil**, **floor** | Rounding (up/down) | $(a) \\to (\\lceil a \\rceil)$ or $(\\lfloor a \\rfloor)$ |
| **Conversions** | **deg** | Convert radians to degrees | $(a) \\to (\\text{degrees})$ |
|  | **rad** | Convert degrees to radians | $(a) \\to (\\text{radians})$ |
| **Combinatorics** | **\!** | Factorial | $(n) \\to (n\!)$ |
|  | **P** | Permutations $P(n, k)$ | $(n, k) \\to P(n, k)$ |
|  | **C** | Combinations $C(n, k)$ | $(n, k) \\to C(n, k)$ |
| **Constants** | **pi**, **e** | Push Constant | $() \\to (\\pi)$ or $(e)$ |
| **Stack/Meta** | **\<\>** | Swap last two items | $(a, b) \\to (b, a)$ |
|  | **c** | Clear the stack | $... \\to ()$ |
|  | **a** | Recall Last Answer | $() \\to (\\text{last result})$ |
|  | **help** | List functions or show usage | Varies |
| **Memory** | **"key" sto** | Store value to key | $(\\text{val}, \\text{key}) \\to ()$ |
|  | **"key" rcl** | Recall value from key | $(\\text{key}) \\to (\\text{val})$ |
| **Display** | **hex**, **bin**, **oct** | Display $a$ in specified base | $(a) \\to (a)$ (with side effect) |

*Note: For the display commands (hex, bin, oct), the number is displayed to the console but remains on the stack.*
