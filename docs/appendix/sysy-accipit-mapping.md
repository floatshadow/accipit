# SysY 结构与 Accipit IR 的对应

## 基本结构

在这一部分我们关注 SysY 语言最基本的结构。

### 局部变量

在 Accipit 中，你能看到两种形式的局部变量：

- 临时变量/虚拟寄存器。
- 在栈上分配的临时。


前一种局部变量是顶层变量 (top level variable)。
取这个名字，是因为他们在 Accipit IR 中定义了一个新的符号，这个符号在定义时被赋的值就是变量的值：

```rust
/// SysY source code:
/// int result = lhs + rhs;
/// `lhs` and `rhs` are local variables.
let %result = add %lhs, %rhs
```

在 RISC-V 和 ARM 之类的 RISC 指令集中，指令集的操作数和目标数都只能是寄存器。
想象一下，假如上面这段 IR 最后翻译到 RISC-V 汇编为 `add t0,t1, t2`，那么 `%result` 对应目标寄存器 `t0`，`%lhs` 和 `%rhs` 分别对应源寄存器 `t1` 和 `t2`。
源操作数 `%lhs` 和 `%rhs` 的值就是局部变量 `lhs` 和 `rhs` 的值，结果 `%result` 就是加法的值，存放了源代码变量 `result` 的值。
这种行为和真实的指令集中的寄存器有些类似，但是和有限数量的物理寄存器不同的是，IR 中的符号可以有无限多，也就是说对应的“局部变量”可以无限多，因此称其为“虚拟寄存器”。


后一种局部变量是取地址变量 (address taken variable)。
取这个名字，是因为他们不像顶层变量有一个新的符号，他们只有局部变量所对应的地址，只能通过 `alloca` 指令创建：

```rust
let %result.var.addr = alloca i64, 1
let %lhs.var.addr = alloca i64, 1
let %rhs.var.addr = alloca i64, 1
```

在这里我们通过 `alloca` 创建 3 个局部变量，由于这三个局部变量都是 `i64` 类型，因此得到的结果 `%result.var.addr``%lhs.var.addr` 和 `%rhs.var.addr` 这三个虚拟寄存器的值都是 `i64*` 类型，代表这三个局部变量的地址。
我们并不知道这三个局部变量叫什么名字，只知道这三个局部变量的地址叫什么名字：`%result.var.addr``%lhs.var.addr` 和 `%rhs.var.addr`。
但是通过地址，我们就能够对这些局部变量读写：

```rust
// read value from the address `%lhs.var.addr`
let %lhs.var.load.0 = load %lhs.var.addr
// write constant int `1` to address `%lhs.var.addr`
let %3 = store 1, %lhs.var.addr
// read again
let %lhs.var.load.1 = load %rhs.var.addr
```

回想 SSA 形式的限制：“出于某种神秘的原因，我们规定每个变量只能在定义的时候被赋值一次”，如果你重复赋值，就会发生错误：

```rust
let %tmp = add 4, 2
let %tmp = add 4, 1 // Error here!
```

但是，SysY 源代码中的局部变量都是可以多次赋值的，`alloca` 指令以及后一种取地址形式的局部变量是为了绕开 SSA 形式的限制，方便你实现“多次赋值”。

比较常见的作法是显式地使用 `alloca` 为所有变量分配栈空间，包括函数参数，当这些变量作为指令的操作数时，先使用一个 `load` 将他们读入临时变量；当这些变量作为指令的目标时，使用一个 `store` 将代表指令结果的临时变量存入地址：

```c
/// `lhs`, `rhs` and `result` are local variables, initialized by constant.
int lhs = 1;
int rhs = 2;
int result = 0;
// fist assignment to `result`
result = lhs + rhs;
// second assignment to `result`
result = result + 1;
```

首先为局部变量分配栈空间：

```rust
let %lhs.addr = alloca i64, 1
let %rhs.addr = alloca i64, 1
let %result.addr = alloca i64, 1
```

然后使用 `store` 指令完成这些局部变量的初始化：

```rust
let %store.lhs = store 1, %lhs.addr
let %store.rhs = store 1, %rhs.addr
let %store.result = store 1, %result.addr
```

`store` 产生的顶层变量/临时变量没什么意义，所以他们的符号可以是匿名的，简化为用数字表示的匿名值：

```rust
let %0 = store 1, %lhs.addr
let %1 = store 1, %rhs.addr
let %2 = store 1, %result.addr
```

第一次赋值，操作数需要局部变量 `lhs` 和 `rhs`，因此需要 `load` 指令读取它们的值：

```rust
let %3 = load %lhs.addr
let %4 = load %rhs.addr
```

同样的，`load` 产生的顶层变量/临时变量只是计算中的中间结果，是临时的值，所以他们的符号最好是匿名的，简化为用数字表示的匿名值。

赋值的目标变量是 `result`，因此需要 `store` 将计算的中间结果写入地址：

```rust
let %5 = add %3, %4
let %6 = store %5, %result.addr
```

第二次赋值，操作数需要局部变量 `result` 和 常数 `1`，因此需要 `load` 指令读取 `lhs` 的值，常数会被内联到加法计算中：

```rust
let %7 = load %result.addr
```

同样的，`load` 产生的顶层变量/临时变量只是计算中的中间结果，是临时的值，所以他们的符号最好是匿名的，简化为用数字表示的匿名值。

赋值的目标变量是 `result`，因此需要 `store` 将计算的中间结果写入地址：

```rust
let %8 = add %7, 1
let %9 = store %8, %result.addr
```

这样我们就在不破坏 SSA 形式限制的情况下，完成了变量的多次赋值，完整的代码清单如下：

```rust
// allocate
let %lhs.addr = alloca i64, 1
let %rhs.addr = alloca i64, 1
let %result.addr = alloca i64, 1
// initialize
let %0 = store 1, %lhs.addr
let %1 = store 1, %rhs.addr
let %2 = store 1, %result.addr
// result = lhs + rhs
let %3 = load %lhs.addr
let %4 = load %rhs.addr
let %5 = add %3, %4
let %6 = store %5, %result.addr
// result = result + 1
let %7 = load %result.addr
let %8 = add %7, 1
let %9 = store %8, %result.addr
```

### 常数

由于 SSA 形式的特性，常量不需要先“复制”给某一个临时的临时变量，而是直接内联在指令中：

例如：

```c
int result = 4 + 2
```

变成：

```rust
let %result = add 4, 2
```

也就是说，指令哪里要用常量，你可以直接把常量插入在那个地方。

### 函数声明和定义

#### 函数声明

一个函数原型，在 Accipit IR 中能翻译到等价的 `fn` 声明：

```c
int bar(int value);
```

变成：

```rust
fn %bar(%value: i64) -> i64;
```

由于没有函数体，函数参数的名字无关紧要，因此你也可以略去参数名，只保留参数类型：

```rust
fn %bar(i64) -> i64;
```

### 控制流结构

类似于汇编，Accipit IR 由一连串指令构成，这些指令一个接一个地顺序执行。
指令按组划分为基本块，每个基本块的终结指令都代表控制流的转移。

#### 简单的 `if-then-else` 分支

让我们来看一个简单的函数，其中包括一个控制流：

```c
int max(int a, int b) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}
```

首先，请铭记，控制流的转移是通过基本块之间的跳转实现的。
基本块内是按顺序执行的指令序列，它们不改变控制流！
只有基本块内的最后一条指令（终结指令）才能改变控制流，进行跳转。
最常见的是条件跳转终结指令 `br`，根据 `%cond` 决定控制流跳转到哪个 label：

```rust
br %cond, label %ifture, label %iffalse
```

和无条件跳转终结指令 `jmp`，直接跳转到某个基本块：

```rust
jmp label %dest
```

```rust
fn max(%a: i64, %b: i64) -> i64 {
%entry:
    let %a.addr = alloca i64, 1
    let %b.addr = alloca i64, 1
    let %0 = store %a, %a.addr
    let %1 = store %b, %b.addr
    let %retval = alloca i64, 1
    let %2 = load %a.addr
    let %3 = load %b.addr
    let %4 = gt %2, %3
    br %4, label %btrue, label %bfalse
%btrue: // preds = %2
    let %5 = store %a, %retval
    br label %end
%bfalse: // preds = %2
    let %6 = store %b, %retval
    br label %end
%end: // preds = %btrue, %bfalse
    let %7 = load %retval
    ret %7
}
```

在上面这个例子中，有 4 个基本块。
第一个是函数的入口，局部变量使用 `alloca` 分配栈空间。
两个参数 `%a` `%b` 使用 `gt` 指令比较大小，结果将作为 `br` 跳转的标志位。
接下来根据不同分支的选择，`%a` 或者 `%b` 会被写入返回值临时变量的地址 `%retval`。
每个分支最后都会有一条无条件跳转 `jmp` 合并控制流到最后的基本块 `%end`，返回值将从 `%retval` 读取并返回。
