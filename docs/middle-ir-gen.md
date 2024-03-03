# Lab 3：中间代码生成


## 背景知识

正如课上可能提到过的，如果没有中间表示 (Intermediate Representation，简称 IR)，n 门语言 m 种硬件平台各自写编译器，就可能需要 n * m 种编译器，但是 n 门语言的前端编译到一个统一的 IR 然后再由 IR 编译到不同的后端，这样只需要 n + m 个“编译器”.

但是，IR 的作用并不仅限于减少编译器开发的工作量，在现代编译器架构下，具体体现在 IR 所指代的对象宽泛化了，现在 IR 通常可以用于泛指“源代码”与“目标平台汇编”之间的各种表示形式，例如抽象语法树、目标无关的中间代码、三地址码风格的类机器代码层等：

- 抽象语法树 AST，树形结构，贴近源代码层，适合做语法糖的展开、构建符号表、类型检查等靠近编程语言的高层级抽象的任务。
同时，也更容易利用这些信息做一些高层次的优化，换句话说他们和程序语言的设计风格息息相关.
例如在 AST 层级，通常是结构化控制流（例如 while loop, for loop, if, switch，函数式风格的可能有 parallel, reduce, yield 等），模式匹配 (pattern match) 就可以被展开为一棵高效的决策树 (decison tree)，减少多余的比较和跳转.
- 目标无关的中间代码（**我们在这里**），常见的设计是线性指令. 由于是平台无关的，设计上通常会考虑屏蔽底层细节；由于考虑适配多语言前端的需要，抛弃了多数高层级信息，更为贴合底层汇编.
在这一层级，一般则为非结构化控制流（例如无条件跳转 jump，分支跳转 branch 等），一般进行例如常量传播、公共子表达式折叠、不变式归纳等与硬件细节无关的优化.
以及控制流分析、数据流分析、别名分析等普适的分析.
- 三地址码风格的类机器代码层，形式上非常接近汇编，甚至可以直接按照汇编指令的格式设计.
这一层非常靠近硬件，优化需要考虑不同指令的延迟、吞吐量、流水线、ABI 等，许多问题是 NP-Hard 的.

我们可以看到，实际上每一层“中间表示”都有各自的特点，依次从高抽象走向低级，适合做的任务也不同，每一层都是一个小型的“编译系统”，因此现代编译器通常会采用多层 IR.

例如 Rust 就曾经在前端增加了一层 [MIR](https://blog.rust-lang.org/2016/04/19/MIR.html)，borrow checker 就在 MIR 层上进行分析：

![Introducing MIR](images/flow.svg)


## 中间代码的定义

本实验的 IR 是类似 LLVM IR 的 partial SSA 形式，具体的规范请参阅 [Accipit IR 规范](appendix/accipit-spec.md).
我们在附录还停供了一些样例：[SysY 结构与 Accipit IR 的对应](appendix/sysy-accipit-mapping.md)，为你演示如何从 SysY 前端的高层级结构是如何翻译到 Accipit IR 的。

下面这段阶乘的样例代码能帮助你实现一个功能正确（虽然显然欠优化的）的中端代码.

源码：

```c
int factorial(int n) {
    if (n == 1) {
        return 1;
    } else {
        int ans = n * factorial(n - 1);
        return ans;
    }
}
```

参考中间代码：
```rust

fn %factorial(#n: i64) -> i64 {
%Lentry:
    // create a stack slot of i64 type as the space of the return value.
    // if n equals 1, store `1` to this address, i.e. `return 1`,
    // otherwise, do recursive call, i.e. return n * factorial(n - 1).
    let %ret.addr: i64* = alloca i64, 1
    // store function parameter on the stack.
    let %n.addr: i64* = alloca i64, 1
    let %4: () = store #n, %n.addr
    // create a slot for local variable ans, uninitialized.
    let %ans.addr: i64* = alloca i64, 1
    // when we need #n, you just read it from %n.addr.
    let %6: i64 = load %n.addr
    // comparison produce an `i8` value.
    let %cmp: i1 = eq %6, 0
    br i1 %cmp, label %Ltrue, label %Lfalse
%Ltrue:
    // retuen value = 1.
    let %10: () = store 1, %ret.addr
    jmp label %Lret
%Lfalse:
    // n - 1
    let %13: i64 = load %n.addr
    let %14: i64 = sub %13, 1
    // factorial(n - 1)
    let %res: i64 = call fn %factorial, %14
    // n
    let %16 = load %n.addr
    // n * factorial(n - 1)
    let %17: i64 = mul %16, %res
    // write local variable `ans`
    let %18: () = store %17, %ans.addr
    // now we meets `return ans`, which means we
    // should first read value from `%ans.addr` and then
    // write it to `%ret.addr`.
    let %19: i64 = load %ans.addr
    let %20: () = store %19, %ret.addr
    jmp label %Lret
%Lret:
    // load return value from %ret.addr
    let %ret.val: i64 = load %ret.addr: i64*
    ret %ret.val: i64
}
```

### 基本块的处理

基本块是划分控制流的边界，基本块内指令有序地线性执行，控制流跳转只存在于基本块之间，这种关系使得基本块之间连成一个有向图，一般称为控制流图 (Contorl Flow Graph，简称 CFG).
例如：`if` 的两个分支分别翻译到两个基本块 `Ltrue` 与 `Lfalse`.

![CFG](images/factorial.svg)

上图是使用 llvm 组件生成的可视化控制流图，你可以使用以下命令获得：

```bash
# clang emit llvm bytecode
$ clang -S -emit-llvm file.c -o file.bc
# convert to dot file
$ opt -dot-cfg -disable-output -enable-new-pm=0 file.bc
Writing '.file.dot'...
# dot render png file
$ dot -Tpng -o test.png .test.dot
```

在 `if` 分支入口前，有一个基本块作为入口，计算 `if` 条件的真假，即 `%cmp`； 
在 `if` 的两个分支结束后，控制流进行了“合并”，处理下一个语句块，进行一个无条件跳转 `br label %ret`，来到了出口基本块 `%ret`.
这是结构化控制流通常的处理方式，你可以将其类推到 `while` 循环，下面是一个示意图：

![while](images/while.svg)

### 局部变量的处理

最简单的实现方式是为所有局部作用域的变量都开辟一块栈上的空间，读局部变量就是 load 对应的地址（IR 中即为 alloca 获取的指针类型的值），写局部变量就是把结果 store 入对应的地址.

如果你还不明白，请看下面的示意图：

![frame](images/frame.svg)


## 语法制导代码生成

下一步我们就要把经过语义检查和推断的语法树转换成中间代码.
基本思路是遍历语法树的节点，然后根据节点的类型生成对应的中间代码.
其核心和语义分析类似，我们要实现一个 translate_X 函数，X 对应表达式，语句等等。

### 表达式生成

正如前面所述，每条指令实际上定义了一个新的变量，因此可以使用指令本身来表示变量，在 Accipit IR 中，值 (value) 包括变量和常数，因此我们先定义 `Value` 类型,：

=== "C"
    C 通常使用 enum + union：

    ```c
    enum value_kind {
        kind_constant_int64,
        Kind_constant_unit,
        // ...
        kind_constant_binary_expression,
        kind_function_call,
        kind_load,
        kind_store,
        // ...
    };

    struct value {
        enum value_kind kind,
        struct type *ty;
        union {
            struct { int number; } constant_int64;
            struct { enum binary_op op, struct value *lhs, *rhs; } binary_expr;
            struct { struct function *callee, struct vector args } function_call;
            struct { struct value* src_addr } load;
            struct { struct value* dest_addr, struct value *value_to_be_stored } store;
        };
    };
    ```

=== "C++"
    C++ 可以使用面对对象实现：

    ```cpp
    class Value {
        Type *ty;
        /*...*/ 
    };

    class ConstantInt : public Value {
        int number;
        /*...*/ 
    };

    class BinaryExpr: public Value {
        Value *lhs, *rhs;
        /*...*/
    };

    class FnCall: public Value {
        Function *callee;
        std::vector<Value *> args;
    };
    ```

=== "OCaml"
    ML 系语言以及 Rust 都支持代数数据类型 (Algebraic Data Type)，可以很方便地定义：

    ```ocaml
    type ValueKinds = ConstantInt of int
        | ConstantUnit
        | BinaryExpr of BinOp * Value * Value
        | FunctionCall of Function * Value list
        | Load of Value
        | Store of Value * Value
    and
    type Value = Type * ValueKinds
    ```

因此 `translate_expr` 函数定义为：

```plaintext
translate_expr(expr, symbol_table, basic_block) -> value
```

也就是说，`translate_expr` 将表达式翻译到中端 IR 的 value.
其中 `symbol_table` 是符号表，通常维护一个 `string -> value` 的映射，虽然上文提到，在类似 SSA 的形式下，变量的名字并不重要，但是在处理局部变量时我们为每个局部变量分配一个栈上的位置，需要记录变量名字到对应 alloca 指令的映射.
重复命名的变量，如在一个语句块里定义的变量和外层的变量重名时，你需要自行处理.

由于 `translate_expr` 是生成线性的指令流，需要传入基本块信息，来指定指令生成在哪个基本块.

面对形如 `expr1 + expr2` 这样的二元表达式，我们递归调用两个子节点的 `translate_expr`，然后生成一条加法指令将他们加起来，最后 `result_value` 将作为 `translate_expr` 的返回值：

```plaintext
lhs_value = translate_expr(expr1, sym_table, basic_block)
rhs_value = translate_expr(expr2, sym_table, basic_block)
result_value = create_binary(lhs, rhs, basic_block)
return result_value
```

上面生成的指令在 IR 中看起来可能像这样，其中 `%2` 是 `translate_expr` 的返回值：

```rust
// lhs value, anonymous
let %0 = .....
// rhs value, anonymous
let %1 = .....
/// result value
let %2 = add %0, %1
```

我们可以将表达式的翻译规则总结如下：



<table>
<colgroup>
<col style="width: 24%" />
<col style="width: 75%" />
</colgroup>
<thead>
<tr class="header">
<th>Expr</th>
<th>Action</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td><code>INT</code></td>
<td><div class="sourceCode" id="cb1"><pre class="sourceCode c"><code class="sourceCode c"><span id="cb1-1"><a href="#cb1-1" aria-hidden="true" tabindex="-1"></a>number <span class="op">=</span> get_number<span class="op">(</span>INT<span class="op">);</span></span>
<span id="cb1-2"><a href="#cb1-2" aria-hidden="true" tabindex="-1"></a><span class="cf">return</span> create_constant_int64<span class="op">(</span>number<span class="op">);</span></span></code></pre></div></td>
</tr>
<tr class="even">
<td><code>ID</code></td>
<td><div class="sourceCode" id="cb2"><pre class="sourceCode c"><code class="sourceCode c"><span id="cb2-1"><a href="#cb2-1" aria-hidden="true" tabindex="-1"></a>addr_of_value <span class="op">=</span> lookup<span class="op">(</span>sym_table<span class="op">,</span> ID<span class="op">);</span>`</span>
<span id="cb2-2"><a href="#cb2-2" aria-hidden="true" tabindex="-1"></a><span class="cf">return</span> create_load<span class="op">(</span>addr_of_value<span class="op">,</span> basic_block<span class="op">);</span></span></code></pre></div></td>
</tr>
<tr class="odd">
<td><code>Expr1 BinOp Expr2</code></td>
<td><div class="sourceCode" id="cb3"><pre class="sourceCode c"><code class="sourceCode c"><span id="cb3-1"><a href="#cb3-1" aria-hidden="true" tabindex="-1"></a>binop <span class="op">=</span> get_binop<span class="op">(</span>BinOp<span class="op">);</span></span>
<span id="cb3-2"><a href="#cb3-2" aria-hidden="true" tabindex="-1"></a>expr1_value <span class="op">=</span> translate_expr<span class="op">(</span>expr1<span class="op">,</span> sym_table<span class="op">);</span></span>
<span id="cb3-3"><a href="#cb3-3" aria-hidden="true" tabindex="-1"></a>expr2_value <span class="op">=</span> translate_expr<span class="op">(</span>expr2<span class="op">,</span> sym_table<span class="op">);</span></span>
<span id="cb3-4"><a href="#cb3-4" aria-hidden="true" tabindex="-1"></a><span class="cf">return</span> create_binary<span class="op">(</span>binop<span class="op">,</span> expr1<span class="op">,</span> expr2<span class="op">,</span> basic_block<span class="op">);</span></span></code></pre></div></td>
</tr>
<tr class="even">
<td><code>MINUS Expr1</code></td>
<td><div class="sourceCode" id="cb4"><pre class="sourceCode c"><code class="sourceCode c"><span id="cb4-1"><a href="#cb4-1" aria-hidden="true" tabindex="-1"></a>zero_value <span class="op">=</span> create_constant_int64<span class="op">(</span><span class="dv">0</span><span class="op">);</span></span>
<span id="cb4-2"><a href="#cb4-2" aria-hidden="true" tabindex="-1"></a>expr1_value <span class="op">=</span> translate_expr<span class="op">(</span>Expr1<span class="op">,</span> sym_table<span class="op">);</span></span>
<span id="cb4-3"><a href="#cb4-3" aria-hidden="true" tabindex="-1"></a><span class="cf">return</span> create_binary<span class="op">(</span>subop<span class="op">,</span> zero_value<span class="op">,</span> expr1_value<span class="op">,</span> basic_block<span class="op">);</span></span></code></pre></div></td>
</tr>
<tr class="odd">
<td><code>Call ID, Args</code></td>
<td><pre><code>function = lookup(sym_table, ID);
args_list = [];
for arg in Args:
  args_list += translate_expr(arg, sym_table, basic_block);
return create_function_call(function, args_list, basic_block);</code></pre></td>
</tr>
</tbody>
</table>

其中 `create_load` `create_binary` `create_function_call` 是生成指令的接口，它们的最后一个参数是基本块 `basic_block`，表示指令在这个基本块中插入，由于基本块中指令是线性的，你可以在基本块中维护一个 `vector`，不断加入指令即可，类似于：

```cpp
void insert_instruction(Instruction *inst, BasicBlock *block) {
    std::vector<Instruction *> &instrs = block.getInstrs();
    instrs.push_bakc(inst);
}
```

### 语句生成

我们定义：

```
translate_stmt(stmt, symbol_table, basic_block) -> exit_basic_block
```

由于语句块可能包含控制流的跳转，但是整个语句块并没有产生值，所以 `translate_stmt` 将返回一个基本块，表示 `stmt` 结束后，控制流将在哪个基本块继续。

条件语句的生成则要复杂些，我们所定义的基本块中间在这里将发挥重要作用.
直觉上来说，If 语句应该生成如下的中间代码：
```c
if (exp) {
    stmt1;
} else {
    stmt2;
}

    let %cond_value = translate_expr(cond)
    br cond, label %true_label, label %false_label
%true_label:
    translate_stmt(stmt1)
    jmp label %exit_label
%false_label:
    translate_stmt(stmt2)
    jmp label %exit_label
%exit_label:
    ...
```

`cond_value` 为真时跳转到 `%true_label`，为假时跳转到 `%false_label`，最后两个基本块的控制流在 `%exit_label` 合并. 
而这也就是 If 语句的翻译流程：

- 生成新的基本块 `true_label`，`false_label` 和 `exit_label`，分别用于条件为真时的跳转，条件为假时的跳转，控制流的合并.
- 调用 `translate_expr` 生成条件表达式的中间代码，传入 `true_label` 和 `false_label` 作为条件为真时和条件为假时的跳转位置.
- 而对于具体的语句，只需要递归调用 `translate_stmt` 即可.
- 把 `true_label` 和 `false_label` 的终结指令 (Terminator) 设置为 `jmp label %exit_label` 完成控制流合并.

其余类型的条件语句本质上是一样的，我们不再一一赘述.
我们总结语句翻译的规则如下：



<table>
<colgroup>
<col style="width: 29%" />
<col style="width: 70%" />
</colgroup>
<thead>
<tr class="header">
<th>Stmt</th>
<th>Action</th>
</tr>
</thead>
<tbody>
<tr class="odd">
<td><code>Expr</code></td>
<td><div class="sourceCode" id="cb1"><pre class="sourceCode c"><code class="sourceCode c"><span id="cb1-1"><a href="#cb1-1" aria-hidden="true" tabindex="-1"></a>translate_expr<span class="op">(</span>expr<span class="op">,</span> sym_table<span class="op">,</span> basic_block<span class="op">);</span></span>
<span id="cb1-2"><a href="#cb1-2" aria-hidden="true" tabindex="-1"></a><span class="cf">return</span> basic_block<span class="op">;</span></span></code></pre></div></td>
</tr>
<tr class="even">
<td><code>ID = Expr</code></td>
<td><div class="sourceCode" id="cb2"><pre class="sourceCode c"><code class="sourceCode c"><span id="cb2-1"><a href="#cb2-1" aria-hidden="true" tabindex="-1"></a>addr_of_value <span class="op">=</span> lookup<span class="op">(</span>sym_table<span class="op">,</span> ID<span class="op">);</span></span>
<span id="cb2-2"><a href="#cb2-2" aria-hidden="true" tabindex="-1"></a>result <span class="op">=</span> translate_expr<span class="op">(</span>Expr<span class="op">,</span> sym_table<span class="op">,</span> basic_block<span class="op">);</span></span>
<span id="cb2-3"><a href="#cb2-3" aria-hidden="true" tabindex="-1"></a>create_store<span class="op">(</span>result<span class="op">,</span> addr_of_value<span class="op">,</span> basic_block<span class="op">);</span></span>
<span id="cb2-4"><a href="#cb2-4" aria-hidden="true" tabindex="-1"></a><span class="cf">return</span> basic_block<span class="op">;</span></span></code></pre></div></td>
</tr>
<tr class="odd">
<td><code>If (Expr) Stmt</code></td>
<td><div class="sourceCode" id="cb3"><pre class="sourceCode c"><code class="sourceCode c"><span id="cb3-1"><a href="#cb3-1" aria-hidden="true" tabindex="-1"></a>exit_basic_block <span class="op">=</span> new_label<span class="op">();</span></span>
<span id="cb3-2"><a href="#cb3-2" aria-hidden="true" tabindex="-1"></a>true_basic_block <span class="op">=</span> new_label<span class="op">();</span></span>
<span id="cb3-3"><a href="#cb3-3" aria-hidden="true" tabindex="-1"></a></span>
<span id="cb3-4"><a href="#cb3-4" aria-hidden="true" tabindex="-1"></a>cond_value <span class="op">=</span> translate_expr<span class="op">(</span>Expr<span class="op">,</span> sym_table<span class="op">,</span> basic_block<span class="op">);</span></span>
<span id="cb3-5"><a href="#cb3-5" aria-hidden="true" tabindex="-1"></a>create_branch<span class="op">(</span>cond<span class="op">,</span> true_basic_block<span class="op">,</span> exit_basic_block<span class="op">,</span> basic_block<span class="op">);</span></span>
<span id="cb3-6"><a href="#cb3-6" aria-hidden="true" tabindex="-1"></a></span>
<span id="cb3-7"><a href="#cb3-7" aria-hidden="true" tabindex="-1"></a>translate_stmt<span class="op">(</span>Stmt<span class="op">,</span> true_label<span class="op">);</span></span>
<span id="cb3-8"><a href="#cb3-8" aria-hidden="true" tabindex="-1"></a>create_jmp<span class="op">(</span>exit_basic_block<span class="op">,</span> true_basic_block<span class="op">);</span></span>
<span id="cb3-9"><a href="#cb3-9" aria-hidden="true" tabindex="-1"></a></span>
<span id="cb3-10"><a href="#cb3-10" aria-hidden="true" tabindex="-1"></a><span class="cf">return</span> exit_basic_block<span class="op">;</span></span></code></pre></div></td>
</tr>
<tr class="even">
<td><code>If (Expr) Stmt1 Else Stmt2</code></td>
<td><div class="sourceCode" id="cb4"><pre class="sourceCode c"><code class="sourceCode c"><span id="cb4-1"><a href="#cb4-1" aria-hidden="true" tabindex="-1"></a>exit_basic_block <span class="op">=</span> new_label<span class="op">();</span></span>
<span id="cb4-2"><a href="#cb4-2" aria-hidden="true" tabindex="-1"></a>true_basic_block <span class="op">=</span> new_label<span class="op">();</span></span>
<span id="cb4-3"><a href="#cb4-3" aria-hidden="true" tabindex="-1"></a>false_basic_block <span class="op">=</span> new_label<span class="op">();</span></span>
<span id="cb4-4"><a href="#cb4-4" aria-hidden="true" tabindex="-1"></a></span>
<span id="cb4-5"><a href="#cb4-5" aria-hidden="true" tabindex="-1"></a>cond_value <span class="op">=</span> translate_expr<span class="op">(</span>Expr<span class="op">,</span> sym_table<span class="op">,</span> basic_block<span class="op">);</span></span>
<span id="cb4-6"><a href="#cb4-6" aria-hidden="true" tabindex="-1"></a>create_branch<span class="op">(</span>cond<span class="op">,</span> true_basic_block<span class="op">,</span> false_basic_block<span class="op">,</span> basic_block<span class="op">);</span></span>
<span id="cb4-7"><a href="#cb4-7" aria-hidden="true" tabindex="-1"></a></span>
<span id="cb4-8"><a href="#cb4-8" aria-hidden="true" tabindex="-1"></a>translate_stmt<span class="op">(</span>Stmt<span class="op">,</span> true_label<span class="op">);</span></span>
<span id="cb4-9"><a href="#cb4-9" aria-hidden="true" tabindex="-1"></a>create_jmp<span class="op">(</span>exit_basic_block<span class="op">,</span> true_basic_block<span class="op">);</span></span>
<span id="cb4-10"><a href="#cb4-10" aria-hidden="true" tabindex="-1"></a></span>
<span id="cb4-11"><a href="#cb4-11" aria-hidden="true" tabindex="-1"></a>translate_stmt<span class="op">(</span>Stmt<span class="op">,</span> false_label<span class="op">);</span></span>
<span id="cb4-12"><a href="#cb4-12" aria-hidden="true" tabindex="-1"></a>create_jmp<span class="op">(</span>exit_basic_block<span class="op">,</span> false_basic_block<span class="op">);</span></span>
<span id="cb4-13"><a href="#cb4-13" aria-hidden="true" tabindex="-1"></a></span>
<span id="cb4-14"><a href="#cb4-14" aria-hidden="true" tabindex="-1"></a><span class="cf">return</span> exit_basic_block<span class="op">;</span></span></code></pre></div></td>
</tr>
<tr class="odd">
<td><code>While (Expr) Stmt</code></td>
<td><div class="sourceCode" id="cb5"><pre class="sourceCode c"><code class="sourceCode c"><span id="cb5-1"><a href="#cb5-1" aria-hidden="true" tabindex="-1"></a>entry_bb <span class="op">=</span> new_label<span class="op">()</span></span>
<span id="cb5-2"><a href="#cb5-2" aria-hidden="true" tabindex="-1"></a>body_bb <span class="op">=</span> new_label<span class="op">()</span></span>
<span id="cb5-3"><a href="#cb5-3" aria-hidden="true" tabindex="-1"></a>exit_bb <span class="op">=</span> new_label<span class="op">()</span></span>
<span id="cb5-4"><a href="#cb5-4" aria-hidden="true" tabindex="-1"></a></span>
<span id="cb5-5"><a href="#cb5-5" aria-hidden="true" tabindex="-1"></a>create_jump<span class="op">(</span>entry_bb<span class="op">,</span> basic_block<span class="op">);</span></span>
<span id="cb5-6"><a href="#cb5-6" aria-hidden="true" tabindex="-1"></a>cond_value <span class="op">=</span> translate_expr<span class="op">(</span>Expr<span class="op">,</span> sym_table<span class="op">,</span> basic_block<span class="op">);</span></span>
<span id="cb5-7"><a href="#cb5-7" aria-hidden="true" tabindex="-1"></a>create_branch<span class="op">(</span>cond<span class="op">,</span> body_bb<span class="op">,</span> exit_bb<span class="op">,</span> entry_bb<span class="op">);</span></span>
<span id="cb5-8"><a href="#cb5-8" aria-hidden="true" tabindex="-1"></a></span>
<span id="cb5-9"><a href="#cb5-9" aria-hidden="true" tabindex="-1"></a>translate_stmt<span class="op">(</span>Stmt<span class="op">,</span> body_bb<span class="op">);</span></span>
<span id="cb5-10"><a href="#cb5-10" aria-hidden="true" tabindex="-1"></a>create_jump<span class="op">(</span>entry_bb<span class="op">,</span> body_bb<span class="op">);</span></span>
<span id="cb5-11"><a href="#cb5-11" aria-hidden="true" tabindex="-1"></a></span>
<span id="cb5-12"><a href="#cb5-12" aria-hidden="true" tabindex="-1"></a><span class="cf">return</span> exit_bb<span class="op">;</span></span></code></pre></div></td>
</tr>
</tbody>
</table>


## 你的任务

在实现 lexer 和 parser 的基础上，将语法树转换为中间代码