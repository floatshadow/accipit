# Accipit IR Specification


## Conventions

Accipit IR 的语法和结构由抽象语法定义。

**注意：** 一般情况下，你的任务并不会涉及 parse Accipit IR，本文档的主要目的是让你熟悉 Accipit IR 的语法并方便你进行后续的 debug 和 test。


### Grammar Notations

以下记号会在定义抽象语法的语法规则时用到：

- 产生式写作 `sym ::= expr1 | expr2 | ... | exprn`，其中 expr 是任意合法的语法表达式。
- 引用其他产生式使用尖括号包裹的该产生式对应的符号，例如 `<sym>`。
- 空字符串使用 `<empty>` 标记。
- 字面量的字符或者字符串使用单引号，例如 `'a'` `'string'`。
- 字面量字符的集合 `[ character-set ]`，表示匹配集合任意一个且仅一个字符，合法的集合包括：单个字符 `'c'`；一个范围内的字符 `'c1'-'c2'` （c1 和 c2 之间的字符，包含两个端点）；两个或多个字符的并 `['c1' 'c2' 'c3' ... 'cn']`。
- `expr *` 表示 0 个或多个符合 expr 字符串的拼接。
- `expr +` 表示 1 个或多个符合 expr 字符串的拼接。
- `expr ?` 表示 0 个或 1 个符合 expr 字符串的拼接。
- `expr1 | expr2` 表示任何符合 expr1 或者 expr2 的字符串。
- `{ expr }` 表示符合 expr 的字符串。
-  `expr1 expr2` 表示两个字符串的拼接，第一个符合 expr1，第二个符合 expr2。

以及一些高级的函数，他们不难使用上面的表达式定义：

- `list(expr)` 表示 0 个或者多个符合 expr 的字符串拼接。
- `nonempty_list(expr)` 表示 1 个或者多个符合 expr 的字符串拼接。
- `separated_list(expr, character)` 表示 0 个或者多个符合 expr 的字符串，其间以字符 character 分隔. 例如符合 `separected_list(expr, ',')` 的字符串有： `<empty>`，`expr`，`expr,expr,expr`。
- `separated_nonempty_list(expr, character)` 含义类似 separated_list，但是不允许空字符串 `<empty>`。

**由于本文档说明按模块划分，所以可能存在部分交叉引用**


### Common Symbols

以下的常用的符号在所有章节中都是共享的：

```
digit      ::=  ['0'-'9']
letter     ::=  ['a'-'z' 'A'-'Z']
ident      ::=  ['%' '#' '@'] [<letter> '-' '_' '.'] [<letter> <digit> '_' '.']*
               | <digit>+
```

digit 定义了数字字符集合。

letter 定义了字母数字集合，简单起见只包含拉丁字母。

ident 定义了标识符 (identifier) 集合，你可以理解为 Accipit IR 内部所使用各种不同结构，值的名字。
具体来说有两种命名习惯：

- 带有名称的变量，例如 `%foo` `@DivisionByZero` ` %a.really.long.identifier`.
- 匿名的变量或者临时变量，通常按出现的顺序使用一个非负整数命名，例如 `%12` `@2` `%0`.

```
int_lit    ::=  '-'? <digit>+
none_lit   ::=  'none'
unit_lit   ::=  'unit'
lit        ::=  <int_lit> | <none_lit> | <unit_lit>
```

除了可以用上述具名或匿名的标识符来引用某个值，Accipit IR 还有常数值。

`int_lit` 定义了 32 位有符号整数字面量，我们只考虑普通的十进制整数的文本形式。

`none_lit` 是两个特殊的符号，用于 offset 指令。

`unit_lit` 是单值类型 unit 的字面量常数。

```
symbol     ::= <ident>
value      ::= <symbol> | lit<>
```

符号 (symbol) 包含所有标识符，代表了中间代码中的变量，包括带有名称的和匿名的临时变量。
而 IR 中合法的值 (value) 既可以是符号，表示对应变量的值；也可以是字面量，表示字面量本身的值。

## Structures

下面我们定义 Accipit IR 的各个结构。

### Types

Accipit IR 中的众多实体按类型区分，类型关乎到程序的合法性、执行的过程。

```
type    ::=   'i32'
            | '()'
            | <type> '*'
            | 'fn' '(' separated_list(<type>, ',') ')' '->' <type>
```

i32，32 位带符号整数。

单值类型 ()，读作 unit，可以理解为空类型 void。

指针类型，由被指的类型 (pointee type) 加上后缀 * 表示。

函数类型，类似于函数声明，例如：
- 加法 add，两个 i32 参数，一个 i32 返回值 `fn(i32, i32) -> i32`。
- 读入，无参数，一个 i32 返回值 `fn() -> i32`。
- 输出，一个 i32 参数，无返回值 `fn(i32) -> ()`。
- `fn(i32*) -> i32*`，接受一个 i32* 参数，返回一个 i32* 类型的返回值。

### Instructions

Accipit IR 的代码由一系列指令 (instruction) 组成。

```
valuebinding   ::= 'let' <symbol> '=' {<binexpr> | <gep> | <fncall> | <alloca> | <load> | <store>}
terminator     ::= <jmp> | <br> | <ret>
```

所有的指令具有两种形式：

- value binding，定义一个变量 `<symbol>`，并给它赋值，即将右侧指令的值赋给左侧 `<symbol>` 代表的变量。
- terminator，不进行任何赋值，标记基本块的终结，对应控制流的跳转。

**注意**：出于某种神秘的原因，我们规定每个变量只能在定义的时候被赋值一次，也就是说，每条指令左侧的变量 `<symbol>` 在对应的作用域内要求是**唯一**的，至于为什么，你可以参考 [附录](quads2ssa.md)。
所以，我们在语法上用 `let` 来暗示这一点。


#### Binary Instructions

```
binop     ::=  'add' | 'sub' | 'mul' | 'div' | 'mod' |
               'and' | 'or' | 'xor' |
               'lt' | 'gt' | 'le' | 'ge' | 'eq' | 'ne'
binexpr   ::=  <binop> <value> ',' <value>
```

##### 说明

数值计算指令操作符中不包含单目运算符，例如 lnot (logic not) 和 neg (numeric negation)，因为他们是多余的：

- 按位取反，`not %src` 等价于 `xor %src: i32, -1`.
- 逻辑取反，`lnot %src` 等价于 `eq %src, 0`.
- 取负数，`neg %src` 等价于 `sub 0, %src`.

这种转换很容易在前端生成中间代码时实现，并且使得这些计算有一个统一的表示形式 (canonical form)，这将有利于后端代码生成.

##### 类型规则

接受两个 i32 类型操作数，返回一个 i32 类型的值.


#### Pointer Instructions

```
gep  ::=  'offset' <type> ',' <symbol> { ',' '[' <value> '<' {<int_lit> | <none_lit>} ']' }+
```

##### 说明

offset 指令的语义比较复杂，它是用于计算地址偏移量的.
在大家比较熟悉的 C 语言中，可能涉及到高维数组、结构体等等寻址比较复杂的结构，这是 offset 尝试解决的问题.

出于简化考虑，accipit IR 都使用普通的指针表示，并使用若干组 size 来标记每个维度上的大小，若干组 index 来标记每个维度上的偏移量：

offset 指令有一个类型标注，用来表明数组中元素类型；
一共有 `2n + 1` 个参数，其中第一个参数是一个指针，表示基地址；
后 `2n` 个参数每两个一组， 每一组的形式为 `[index < size]` 其中 index 表示该维度上的偏移量，size 表示该维度的大小.

例如 C 语言中声明数组 `int g[3][2][5]`，访问元素 `g[x][y][z]` 时，对应的 offset 指令为 `offset i32, %g.addr: i32*, [x < 3], [y < 2], [z < 5]`。

当然，可能会出现高位数组有一维不知道大小或者单个指针偏移的情况，在这种情况下，对应的维度使用 none 标记：
- 二维数组 `int g[][5]` 访问 `g[x][y]`，`offset i32, %g.addr: i32*, [x < none], [y < 5]`.
- 单个指针 `int *p` 访问 `p + 10`，`offset i32, %g.addr: i32*, [10 < none]`.

为什么要有 size 这个参数作为一个下标的上界？
为了你方便处理，我们在类型中舍弃了高维数组，因为数组类型在后端代码生成时处理相对比较麻烦，但是在前端处理这些信息相对容易。
此外为了方便检查可能的下标越界错误，解释器需要 bound 信息标注。

##### 类型规则

假设基地址变量 `<symbol>` 是指针类型 T*，则要求标注的数组中元素类型 `<type>` 必须为 T。

假设，有 `n` 组偏移量和维度 `[index_0 < size_0], [index_1 < size_1], ... [index_{n-1} < size_{n-1}]`，
为了保证语义的合法性，只有 `size_0` 可以为 `none`，所有的 size 如果不是 `none` 则必须是一个正整数字面量。


#### Function Call Instructions

```
fncall ::= 'call' <symbol> list({',' <value>})
```

##### 说明

fncall 指令进行函数调用，符号 `<symbol>` 必须是被调用的函数，后跟一个参数列表.

##### 类型规则

如果被调用的函数 `<symbol>` 是 `fn(T_1, T_2, ..., T_{n-1}) -> T_0` 类型，那么参数列表的参数必须依次为 `T_1` `T_2` ... `T_{n-1}` 类型，这条指令讲返回一个 `T_0` 类型的值。

假设 `<symbol>` 没有参数，即为 `fn() -> T` 类型，那么可以不写参数列表，即 `let %ret = call <symbol>`。

假设 `<symbol>` 的返回值是 unit，即 `fn(T_1, T_2, ...) -> ()` 类型，那么返回值也为 unit 类型。

#### Memory Instructions

```
alloca ::= 'alloca' <type> ',' <int_lit>
load   ::= 'load' <symbol>
store  ::= 'store' <value> ',' <symbol> 
```

##### 说明

alloca 指令的作用是为局部变量开辟栈空间，并获得一个指向 `<type>` 类型，长度为 `<int_lit>` 的指针。
可以理解为，在栈上定义一个数组 `<type>[<int_lit>]`，并获取数组首元素的地址。
或者类比 C 代码 `int *a = (int *)malloc(100 * sizeof(int))`， 对应 `let %a: i32* = alloc i32, 100`.

load 指令接受一个指针类型 T* 的符号，返回一个 T 类型的值.

store 指令接受一个类型 T 的值，将其存入一个 T* 类型的符号，并返回 unit 类型的值.

#### Terminator Instructions

```
br    ::=  'br' <value> ',' 'label' <ident> ',' 'label' <ident>
jmp   ::=  'jmp' 'label' <ident>
ret   ::=  'ret' <value> 
```

##### 说明

br 进行条件跳转，接受的 `<value>` 应当是 i32 类型.
若为 true，跳转到第一个 `<label>` 标记的基本块起始处执行；
若为 false，跳转到第二个 `<label>` 标记的基本块起始处执行.

jmp 进行无条件跳转，跳转到 `<label>` 标记的基本块起始处执行.

ret 进行函数范围，并将 `<value>` 作为返回值，返回值的类型应当与函数签名一致.


### Functions

函数是一段连续的指令序列

```
plist ::= separated_list({<ident> ':' <type>}, ',')
fun   ::= 'fn' <ident> '(' <plist> ')' '->' <type> {';' | <body>}
```

函数包含函数头以及可选的函数体。
如果没有函数体，则以一个分号 `;` 结尾，例如 `fn getchar() -> i32;`

函数头以关键字 `fn` 开头，包含函数命令和参数列表和返回值，参数列表必须显式地标出每个参数的类型。

```
body  ::= '{' <bb>+ '}'
label ::= <ident> ':'
bb    ::= <label> <instr>* <terminator>
```

函数体由一系列基本块 (basic block) 组成，每个基本块有一个标号 (label) 和基本块中的指令序列，其中最后一条指令必须是 terminator.

下面是一个阶乘函数的例子：
```
fn %factorial(%n: i32) -> i32 {
%Lentry:
    /* Create a stack slot of i32 type as the space of the return value.
     * if n equals 1, store `1` to this address, i.e. `return 1`,
     * otherwise, do recursive call, i.e. return n * factorial(n - 1).
     */
    let %ret.addr = alloca i32, 1
    let %cmp = eq %n, 0
    br %cmp, label %Ltrue, label %Lfalse
%Ltrue:
    let %6 = store 1, %ret.addr
    jmp label %Lret
%Lfalse:
    let %9 = sub %n, 1
    let %res = call %factorial, %9
    let %11 = mul %9, %res
    let %12 = store %11, %ret.addr
    jmp label %Lret
%Lret:
    let %ret.val = load %ret.addr
    ret %ret.val
}
```

### Globals

```
global ::= <symbol> ':' 'region' <type> <int_lit>
```

##### 说明

声明全局变量 `<symbol>`，变量具有 `<type>` 类型和 `<int_lit>`。

##### 类型规则

和 alloca 类似，`<type>` 是全局变量可存储的元素类型，`<symbol>` 的类型是对应的指针类型 `<type> *`。

例如：

```
%a : region i32, 2
```

则 `%a` 为 `i32*` 类型，所指向的地址能存放 2 个 i32。

## Execution

TBD


## Acknowledgement

IR 的设计参考了以下课程与资料：

- [北京大学编译原理课程](https://pku-minic.github.io/online-doc/#/) 的 Koopa IR.
- [LLVM 项目](https://llvm.org/docs/LangRef.html) 的 IR 设计.
- [SyOC](https://github.com/waterlens/syoc) 的 IR 设计，感谢 @waterlens 的热心帮助.
