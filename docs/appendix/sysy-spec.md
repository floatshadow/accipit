# SysY 语言规范

SysY语言是[全国大学生计算机系统能力大赛](https://compiler.educg.net/#/)中编译系统设计赛要实现的编程语言.

你可以在[这里](https://gitlab.eduxiji.net/nscscc/compiler2023/-/blob/master/SysY2022%E8%AF%AD%E8%A8%80%E5%AE%9A%E4%B9%89-V1.pdf)找到正式的 SysY 的文法和语义约束.
本实验所使用的 SysY 语言和官方定义略有不同. **我们对SysY语言做了一些修改**, 具体如下: 

- 删除了 `float` 类型, 即不需要实现浮点数类型. 
- 删除了 `ConstDecl`，即不需要实现常量声明。同时数组仅支持整数字面量声明维度。
- 删除了 `InitVal` 的嵌套, 即不需要实现数组的初始化.
- 删除了十进制之外的整数字面量.


## 文法定义

SysY 语言的文法采用扩展的 Backus 范式 (EBNF, Extended Backus-Naur Form) 表示, 其中:

* 符号 `[...]` 表示方括号内包含的项可被重复 0 次或 1 次.
* 符号 `{...}` 表示花括号内包含的项可被重复 0 次或多次.
* 终结符是由双引号括起的串, 或者是 `IDENT`, `INT_CONST` 这样的大写记号. 其余均为非终结符.

SysY 语言的文法表示如下, `CompUnit` 为开始符号:

```ebnf
CompUnit      ::= [CompUnit] (Decl | FuncDef);

BType         ::= "int";
Decl          ::= VarDecl;


VarDecl       ::= BType VarDef {"," VarDef} ";";
VarDef        ::= IDENT "=" InitVal
                | IDENT {"[" INT_CONST "]"};
InitVal       ::= Exp;

FuncDef       ::= FuncType IDENT "(" [FuncFParams] ")" Block;
FuncType      ::= "void" | "int";
FuncFParams   ::= FuncFParam {"," FuncFParam};
FuncFParam    ::= BType IDENT ["[" "]" {"[" INT_CONST "]"}];

Block         ::= "{" {BlockItem} "}";
BlockItem     ::= Decl | Stmt;
Stmt          ::= LVal "=" Exp ";"
                | Exp ";"
                | Block
                | "if" "(" Exp ")" Stmt ["else" Stmt]
                | "while" "(" Exp ")" Stmt
                | "break" ";"
                | "continue" ";"
                | "return" [Exp] ";";

Exp           ::= LOrExp;
LVal          ::= IDENT {"[" Exp "]"};
PrimaryExp    ::= "(" Exp ")" | LVal | Number;
Number        ::= INT_CONST;
UnaryExp      ::= PrimaryExp | IDENT "(" [FuncRParams] ")" | UnaryOp UnaryExp;
UnaryOp       ::= "+" | "-" | "!";
FuncRParams   ::= Exp {"," Exp};

MulExp        ::= UnaryExp | MulExp ("*" | "/" | "%") UnaryExp;
AddExp        ::= MulExp | AddExp ("+" | "-") MulExp;

RelExp        ::= AddExp | RelExp ("<" | ">" | "<=" | ">=") AddExp;
EqExp         ::= RelExp | EqExp ("==" | "!=") RelExp;
LAndExp       ::= EqExp | LAndExp "&&" EqExp;
LOrExp        ::= LAndExp | LOrExp "||" LAndExp;
```

## SysY 语言的终结符特征

### 标识符

SysY 语言中标识符 `IDENT` (identifier) 的规范如下:

```ebnf
identifier ::= identifier-nondigit
             | identifier identifier-nondigit
             | identifier digit;
```

其中, `identifier-nondigit` 为下划线, 小写英文字母或大写英文字母; `digit` 为数字 0 到 9.

关于其他信息, 请参考 [ISO/IEC 9899](http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1124.pdf) 第 51 页关于标识符的定义.

对于同名**标识符**, SysY 中有以下约定:

* 全局变量和局部变量的作用域可以重叠, 重叠部分局部变量优先.
* 同名局部变量的作用域不能重叠.
* 变量名可以和函数名相同 (这里可以将函数视为全局变量).
* 变量名不可以和保留字 (keyword) 相同.

### 数值常量

SysY 语言中数值常量可以是整型数 `INT_CONST` (integer-const), 其规范如下:

```ebnf
integer-const ::= digit { digit };
```

数值常量的范围为 $[0, 2^{31} - 1]$, 不包含负号.

### 注释

SysY 语言中注释的规范与 C 语言一致, 如下:

* 单行注释: 以序列 `//` 开始, 直到换行符结束, 不包括换行符.
* 多行注释: 以序列 `/*` 开始, 直到第一次出现 `*/` 时结束, 包括结束处 `*/`.

关于其他信息, 请参考 [ISO/IEC 9899](http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1124.pdf) 第 66 页关于注释的定义.

## 语义约束

下面, 我们进一步给出 SysY 语言的语义约束.

### 编译单元

```ebnf
CompUnit ::= [CompUnit] (Decl | FuncDef);
Decl ::=  VarDecl;
```

1. 一个 SysY 程序由单个文件组成, 文件内容对应 EBNF 表示中的 `CompUnit`. 在该 `CompUnit` 中, 必须存在且仅存在一个标识为 `main`, 无参数, 返回类型为 `int` 的 `FuncDef` (函数定义). `main` 函数是程序的入口点.
2. `CompUnit` 的顶层变量/常量声明语句 (对应 `Decl`), 函数定义 (对应 `FuncDef`) 都不可以重复定义同名标识符 (`IDENT`), 即便标识符的类型不同也不允许.
3. `CompUnit` 的变量/常量/函数声明的作用域从该声明处开始, 直到文件结尾.

### 变量定义

```ebnf
VarDef ::= IDENT {"[" INT_CONST "]"}
         | IDENT {"[" INT_CONST "]"} "=" InitVal;
```

1. `VarDef` 用于定义变量. 当不含有 `=` 和初始值时, 其运行时实际初值未定义.
2. `VarDef` 的数组维度和各维长度的定义部分不存在时, 表示定义单个变量; 存在时, 表示定义多维数组


### 函数形参与实参

```ebnf
FuncFParam ::= BType IDENT ["[" "]" {"[" INT_CONST "]"}];
FuncRParams ::= Exp {"," Exp};
```

1. `FuncFParam` 定义函数的一个形式参数. 当 `IDENT` 后面的可选部分存在时, 表示定义数组类型的形参.
2. 当 `FuncFParam` 为数组时, 其第一维的长度省去 (用方括号 `[]` 表示), 而后面的各维则需要用表达式指明长度, 其长度必须是常量.
3. 函数实参的语法是 `Exp`. 对于 `int` 或 `bool` 类型的参数, 遵循按值传递的规则; 对于数组类型的参数, 形参接收的是实参数组的地址, 此后可通过地址间接访问实参数组中的元素.
4. 对于多维数组, 我们可以传递其中的一部分到形参数组中. 例如, 若存在数组定义 `int a[4][3]`, 则 `a[1]` 是包含三个元素的一维数组, `a[1]` 可以作为实参, 传递给类型为 `int[]` 的形参.

### 函数定义

```ebnf
FuncDef ::= FuncType IDENT "(" [FuncFParams] ")" Block;
```

1. `FuncDef` 表示函数定义. 其中的 `FuncType` 指明了函数的返回类型.
    * 当返回类型为 `int` 或 `bool` 时, 函数内的所有分支都应当含有带有 `Exp` 的 `return` 语句. 不含有 `return` 语句的分支的返回值未定义.
    * 当返回值类型为 `void` 时, 函数内只能出现不带返回值的 `return` 语句.
2. `FuncDef` 中形参列表 (`FuncFParams`) 的每个形参声明 (`FuncFParam`) 用于声明 `int` 类型的参数, 或者是元素类型为 `int` 的多维数组. `FuncFParam` 的语义参见前文.

### 语句块

```ebnf
Block ::= "{" {BlockItem} "}";
BlockItem ::= Decl | Stmt;
```

1. `Block` 表示语句块. 语句块会创建作用域, 语句块内声明的变量的生存期在该语句块内.
2. 语句块内可以再次定义与语句块外同名的变量或常量 (通过 `Decl` 语句), 其作用域从定义处开始到该语句块尾结束, 它覆盖了语句块外的同名变量或常量.

### 语句

```ebnf
Stmt ::= LVal "=" Exp ";"
       | [Exp] ";"
       | Block
       | "if" "(" Exp ")" Stmt ["else" Stmt]
       | "while" "(" Exp ")" Stmt
       | "break" ";"
       | "continue" ";"
       | "return" [Exp] ";";
```

1. `Stmt` 中的 `if` 型语句遵循就近匹配的原则.
2. 单个 `Exp` 可以作为 `Stmt`. `Exp` 会被求值, 所求的值会被丢弃.

### 左值表达式

```ebnf
LVal ::= IDENT {"[" Exp "]"};
```

1. `LVal` 表示具有左值的表达式, 可以为变量或者某个数组元素.
2. 当 `LVal` 表示数组时, 方括号个数必须和数组变量的维数相同 (即定位到元素). 若 `LVal` 表示的数组作为数组参数参与函数调用, 则数组的方括号个数可以不与维数相同.
3. 当 `LVal` 表示单个变量时, 不能出现后面的方括号.

### 表达式

```ebnf
Exp ::= LOrExp;
...
```

1. `Exp` 在 SysY 中代表 `int` 型表达式. 当 `Exp` 出现在表示条件判断的位置时 (例如 `if` 和 `while`), 表达式值为 0 时为假, 非 0 时为真.
2. 对于 `LOrExp`, 当其左右操作数有任意一个非 0 时, 表达式的值为 1, 否则为 0; 对于 `LAndExp`, 当其左右操作数有任意一个为 0 时, 表达式的值为 0, 否则为 1. 上述两种表达式均满足 C 语言中的短路求值规则.
3. `LVal` 必须是当前作用域内, 该 `Exp` 语句之前曾定义过的变量或常量. 赋值号左边的 `LVal` 必须是变量.
4. 函数调用形式是 `IDENT "(" FuncRParams ")"`, 其中的 `FuncRParams` 表示实际参数. 实际参数的类型和个数必须与 `IDENT` 对应的函数定义的形参完全匹配.
5. SysY 中算符的优先级与结合性与 C 语言一致, 上一节定义的 SysY 文法中已体现了优先级与结合性的定义.
