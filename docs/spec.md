# Structure


## Conventions

Accipit IR 的语法和结构由抽象语法定义.

**注意：** 一般情况下，你的任务并不会涉及 parse Accipit IR，本文档的主要目的是让你熟悉 Accipit IR 的语法并方便你进行后续的 debug 和 test. 


### Grammar Notations

以下记号会在定义抽象语法的语法规则时用到：
- 产生式写作 `sym ::= expr1 | expr2 | ... | exprn`，其中 expr 是任意合法的语法表达式.
- 引用其他产生式使用尖括号包裹的该产生式对应的符号，例如 `<sym>`.
- 空字符串使用 `<empty>` 标记.
- 字面量的字符或者字符串使用单引号，例如 `'a'` `'string'`.
- 字面量字符的集合 `[ character-set ]`，表示匹配集合任意一个且仅一个字符，合法的集合包括：单个字符 `'c'`；一个范围内的字符 `'c1'-'c2'` （c1 和 c2 之间的字符，包含两个端点）；两个或多个字符的并 `['c1' 'c2' 'c3' ... 'cn']`
- `expr *` 表示 0 个或多个符合 expr 字符串的拼接
- `expr +` 表示 1 个或多个符合 expr 字符串的拼接
- `expr ?` 表示 0 个或 1 个符合 expr 字符串的拼接
- `expr1 | expr2` 表示任何符合 expr1 或者 expr2 的字符串
- `{ expr }` 表示符合 expr 的字符串
-  `expr1 expr2` 表示两个字符串的拼接，第一个符合 expr1，第二个符合 expr2

以及一些高级的函数，他们不难使用上面的表达式定义：
- `list(expr)` 表示 0 个或者多个符合 expr 的字符串拼接
- `nonempty_list(expr)` 表示 1 个或者多个符合 expr 的字符串拼接
- `separated_list(expr, character)` 表示 0 个或者多个符合 expr 的字符串，其间以字符 character 分隔. 例如符合 `separected_list(expr, ',')` 的字符串有： `<empty>`，`expr`，`expr,expr,expr`.
- `separated_nonempty_list(expr, character)` 含义类似 separated_list，但是不允许空字符 

**由于本文档说明按模块划分，所以可能存在部分交叉引用**


### Common Symbols

以下的常用的符号在所有章节中都是共享的：

```
digit      ::=  ['0'-'9']
letter     ::=  ['a'-'z' 'A'-'Z']
ident      ::=   [<letter> '-' '_' '.'] [<letter> <digit> '_' '.']*
               | <digit>+
```

digit 定义了数字字符集合.

letter 定义了字母数字集合，简单起见只包含拉丁字母.

ident 定义了标识符 (identifier) 集合，你可以理解为 Accipit IR 内部所使用各种不同结构，值的名字.
具体来说有两种命名习惯：
- 带有名称的值，命名习惯与 name 类似，但是允许含有 '.' '-' 等符号，例如 `%foo` `@DivisionByZero` ` %a.really.long.identifier`.
- 匿名的值，这类值通常用于表示编译器生成的临时变量，通常按出现的顺序使用一个非负整数命名，例如 `%12` `@2` `%0`.

```
vident    ::=  '%' <ident>
pident    ::=  '#' <ident>
gident    ::=  '@' <ident>
```

为了避免与保留字 (reserved word) 冲突，以及与 name 相区分，实际使用时我们通常会给 ident 加上 '%' '@' '#' 等前缀以示区分：
- `%` 前缀的标识符用于指令 (instruction) 定义的符号；
- `#` 前缀的标识符用于函数参数列表 (paramter list) 的符号；
- `@` 前缀的标识符用于全局值的符号.

```
int_lit    ::=  '-'? <digit>+
bool_lit   ::=  'true' | 'false'
null_lit   ::=  'null'
none_lit   ::=  'none'
```

除了可以用上述具名或匿名的标识符来引用某个值，Accipit IR 还有常数值.
int_lit 定义了 32 位有符号整数字面量，我们只考虑普通的十进制整数语法.
特别地，字符串 true 或者 false 可以用于 1 bit 整数 (Boolean) 的字面量，分别指代 1 和 0.

null_lit 与 none_lit 是两个特殊的符号，前者用于表示空指针常量 (null pointer)，后者用于 offset 指令.

```
symbol     ::= {<vident> | <pident> | <gident>} {':' <type>}?
value      ::= <symbol> | <int_lit> | <bool_lit>
```

符号 (symbol) 包含所有标识符们，
而 IR 中合法的值 (value) 既可以是符号所绑定的值，也可以是字面量.
为了方便你阅读（尽管这可能显得很多余，确信），所有的标识符都有可选的类型标注.

### Types

accipit IR 中的众多实体按类型区分，类型关乎到程序的合法性、执行的过程.

```
type    ::=   'i32'
            | 'i1'
            | '()'
            | <type> '*'
            | {<type> '->'}+ <type>
```

i32，32 位带符号整数.
i1，1 位整数.

单值类型 ()，读作 unit，可以理解为空类型 void.

指针类型，由被指的类型 (pointee type) 加上后缀 * 表示.

函数类型，采用柯里化的写法，例如：
- 加法 add，两个 i32 参数，一个 i32 返回值 `i32 -> i32 -> i32`
- 读入，无参数，一个 i32 返回值 `() -> i32`
- 输出，一个 i32 参数，无返回值 `i32 -> ()`

### Instructions

Accipit IR 的代码由一系列指令 (instruction) 组成.
Accipit IR 的计算模型基于命令式语言的寄存器机 (register machine).

```
valuebinding   ::= 'let' <vident>  '=' {<binexpr> | <gep> | <fncall> | <alloca> | <load> | <store>}
terminator     ::= <jmp> | <br> | <ret>
```

所有的指令具有两种形式：
- value binding，即将标识符绑定在某个值上.
- terminator，不进行任何绑定，标记基本块的终结，对应控制流的跳转.


出于其中间代码的地位考虑，我们仍然保留了一部分类型信息.
例如加法指令 `let %res:i32 = add %0: i32, %1: i32`.
可能有某些值是空类型，`let %no.result: () = store 1, %addr: i32*`.

#### Numeric Instructions

```
binop     ::=  'add' | 'sub' | 'mul' | 'div' | 'mod' |
               'and' | 'or' | 'xor' |
               'lt' | 'gt' | 'le' | 'ge' | 'eq' | 'ne'
binexpr   ::=  <binop> <value> ',' <value>
```

数值计算指令操作符中不包含单目运算符，例如 lnot (logic not) 和 neg (numeric negation)，因为他们是多余的：
- 按位取反，`not %src: i32` 等价于 `xor %src: i32, -1`.
- 逻辑取反，`lnot %src: i8` 等价于 `eq %src: i8, false`.
- 取负数，`neg %src: i32` 等价于 `sub 0, %src: i32`.
这种转换很容易在前端生成中间代码时实现，并且使得这些计算有一个统一的表示形式 (canonical form)，这将有利于后端代码生成.

### Pointer Instructions

```
gep  ::=  vident type '=' 'offset' type uident ('[' uident '<' (int_lit | none_lit) ']')+
```

offset 指令的语义比较复杂，它是用于计算地址偏移量的.
在大家比较熟悉的 C 语言中，可能涉及到高维数组、结构体等.

出于简化考虑，accipit IR 都使用普通的指针表示，并使用若干组 size 来标记每个维度上的大小，若干组 index 来标记每个维度上的偏移量：
offset 指令有一个类型标注，用来表明数组中元素类型；
一共有 `2n + 1` 个参数，其中第一个参数是一个指针，表示基地址；
后 `2n` 个参数每两个一组， 每一组的形式为 `[index < size]` 其中 index 表示该维度上的偏移量，size 表示该维度的大小.

例如 C 语言中声明数组 `int g[3][2][5]`，访问元素 `g[x][y][z]` 时，对应的 offset 指令为 `offset i32, %g.addr: i32*, [x < 3], [y < 2], [z < 5]`.
当然，可能会出现高位数组有一维不知道大小或者单个指针偏移的情况，在这种情况下，对应的维度使用 none 标记：
- 二维数组 `int g[][5]` 访问 `g[x][y]`，`offset i32, %g.addr: i32*, [x < none], [y < 5]`.
- 单个指针 `int *p` 访问 `p + 10`，`offset i32, %g.addr: i32*, [10 < none]`.

为什么要有 size 这个参数作为一个下标的上界？
为了你方便处理，我们在类型中舍弃了高维数组，数组类型在后端代码生成时处理相对比较麻烦，但是在前端处理这些信息相对容易.
此外为了方便检查可能的下标越界错误，解释器需要 bound 信息标注.

#### Function Call Instructions

```
fncall ::= 'call' 'fn' <symbol> list({',' <value>})
```

fncall 指令进行函数调用，符号 symbol 必须是函数类型，参数列表的中值的数量和类型应当和函数参数一致.

#### Memory Instructions

```
alloca ::= 'alloca' <type> ',' <int_lit>
load   ::= 'load' <symbol>
store  ::= 'store' <value> ',' <symbol> 
```

alloca 指令的作用是为局部变量开辟栈空间，并获得一个指向 type 类型，长度为 int_lit 的指针.
C 代码 `int *a = (int *)malloc(100 * sizeof(int))` 对应 `let %a: i32* = alloc i32, 100`.

load 指令接受一个指针类型 T* 的符号，返回一个 T 类型的值.

store 指令接受一个类型 T 的值，将其存入一个 T* 类型的符号，并返回 unit 类型的值.

#### Terminator Instructions

```
br    ::=  'br' <value> ',' 'label' <ident> ',' 'label' <ident>
jmp   ::=  'jmp' 'label' <ident>
ret   ::=  'ret' <value> 
```

br 进行条件跳转，接受的 value 应当是 i8 类型.
若为 true，跳转到第一个 label 标记的基本块起始处执行；
若为 false，跳转到第二个 label 标记的基本块起始处执行.

jmp 进行无条件跳转，跳转到 label 标记的基本块起始处执行.

ret 进行函数范围，并将 value 作为返回值，返回值的类型应当与函数签名一致.

### Functions

函数是一段连续的指令序列

```
plist ::= separated_list({<pident> ':' <type>}, ',')
fun   ::= 'fn' <vident> '(' <plist> ')' '->' <type? {';' | <body>}
```

函数包含关键字 fn、函数标识符、参数列表、类型以及可选的函数体.

如果没有函数体，则以一个分号 `;` 结尾.

```
body  ::= '{' <bb>+ '}'
label ::= <ident> ':'
bb    ::= <label> <instr>* <terminator>
```

函数体由一系列基本块 (basic block) 组成，每个基本块有一个标号 (label) 和基本块中的指令序列，其中最后一条指令必须是 terminator.

下面是一个阶乘函数的例子：
```
fn factorial(#n: i32) -> i32 {
Lentry:
    /* Create a stack slot of i32 type as the space of the return value.
     * if n equals 1, store `1` to this address, i.e. `return 1`,
     * otherwise, do recursive call, i.e. return n * factorial(n - 1).
     */
    %ret.addr: i32* = alloca i32, 1
    %cmp: i1 = eq #n: i32, 0
    br i1 %cmp, label Lt, Lfalse
Ltrue:
    %6 () = store 1, %ret.addr
    jmp label Lret
Lfalse:
    %9: i32 = sub #n: i32, 1
    %res: i32 = call fn factorial, %9
    %11: i32 = mul %9, %res
    %12 () = store %11: i32, %ret.addr
    jmp label Lret
Lret:
    %ret.val: i32 = load %ret.addr: i32*
    ret i32 %ret.val
}
```

### Globals

```
global ::= <gident> ':' <type> <int_lit>
```

# Execution

TBD


# Acknowledgement

IR 的设计参考了以下课程与资料：
- [北京大学编译原理课程](https://pku-minic.github.io/online-doc/#/) 的 Koopa IR.
- [LLVM 项目](https://llvm.org/docs/LangRef.html) 的 IR 设计.
- [SyOC](https://github.com/waterlens/syoc) 的 IR 设计，感谢 @杨汝清 学长的热心帮助.