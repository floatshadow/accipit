# SysY 语言规范

SysY语言是[全国大学生计算机系统能力大赛](https://compiler.educg.net/#/)中编译系统设计赛要实现的编程语言.

你可以在[这里](https://gitlab.eduxiji.net/nscscc/compiler2023/-/blob/master/SysY2022%E8%AF%AD%E8%A8%80%E5%AE%9A%E4%B9%89-V1.pdf)找到正式的 SysY 的文法和语义约束.
本实验所使用的 SysY 语言和官方定义略有不同. **我们对SysY语言做了一些修改**, 具体如下: 

- 删除了 `float` 类型, 即不需要实现浮点数类型. 
- 增加了 `bool` 类型, 且不支持 `bool` 类型和 `int` 类型之间的隐式转换. 
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

BType         ::= "int" ｜ "bool";
Decl          ::= ConstDecl | VarDecl;
ConstDecl     ::= "const" BType ConstDef {"," ConstDef} ";";
ConstDef      ::= IDENT "=" ConstInitVal
                | IDENT "[" ConstExp "]" {"[" ConstExp "]"};
ConstInitVal  ::= ConstExp;

VarDecl       ::= BType VarDef {"," VarDef} ";";
VarDef        ::= IDENT "=" InitVal
                | IDENT "[" ConstExp "]" {"[" ConstExp "]"};
InitVal       ::= Exp;

FuncDef       ::= FuncType IDENT "(" [FuncFParams] ")" Block;
FuncType      ::= "void" | "int" | "bool";
FuncFParams   ::= FuncFParam {"," FuncFParam};
FuncFParam    ::= BType IDENT ["[" "]" {"[" ConstExp "]"}];

Block         ::= "{" {BlockItem} "}";
BlockItem     ::= Decl | Stmt;
Stmt          ::= LVal "=" Exp ";"
                | [Exp] ";"
                | Block
                | "if" "(" Exp ")" Stmt ["else" Stmt]
                | "while" "(" Exp ")" Stmt
                | "break" ";"
                | "continue" ";"
                | "return" [Exp] ";";

Exp           ::= LOrExp;
LVal          ::= IDENT {"[" Exp "]"};
PrimaryExp    ::= "(" Exp ")" | LVal | Number | Boolean;
Number        ::= INT_CONST;
Boolean       ::= "true" | "false"; 
UnaryExp      ::= PrimaryExp | IDENT "(" [FuncRParams] ")" | UnaryOp UnaryExp;
UnaryOp       ::= "+" | "-" | "!";
FuncRParams   ::= Exp {"," Exp};

MulExp        ::= UnaryExp | MulExp ("*" | "/" | "%") UnaryExp;
AddExp        ::= MulExp | AddExp ("+" | "-") MulExp;

RelExp        ::= AddExp | RelExp ("<" | ">" | "<=" | ">=") AddExp;
EqExp         ::= RelExp | EqExp ("==" | "!=") RelExp;
LAndExp       ::= EqExp | LAndExp "&&" EqExp;
LOrExp        ::= LAndExp | LOrExp "||" LAndExp;
ConstExp      ::= Exp;
```

其中, 各符号的含义如下:

| 符号        | 含义          | 符号          | 含义          |
| ---         | ---           | ---           | ---           |
| CompUnit    | 编译单元      | Decl          | 声明          |
| ConstDecl   | 常量声明      | BType         | 基本类型      |
| ConstDef    | 常数定义      | ConstInitVal  | 常量初值      |
| VarDecl     | 变量声明      | VarDef        | 变量定义      |
| InitVal     | 变量初值      | FuncDef       | 函数定义      |
| FuncType    | 函数类型      | FuncFParams   | 函数形参表    |
| FuncFParam  | 函数形参      | Block         | 语句块        |
| BlockItem   | 语句块项      | Stmt          | 语句          |
| Exp         | 表达式        | LVal          | 左值表达式    |
| PrimaryExp  | 基本表达式    | Number        | 数值          |
| UnaryExp    | 一元表达式    | UnaryOp       | 单目运算符    |
| FuncRParams | 函数实参表    | MulExp        | 乘除模表达式  |
| AddExp      | 加减表达式    | RelExp        | 关系表达式    |
| EqExp       | 相等性表达式  | LAndExp       | 逻辑与表达式  |
| LOrExp      | 逻辑或表达式  | ConstExp      | 常量表达式    |

需要注意的是:

* `Exp`: SysY 中表达式的类型为 `int` 或 `bool` 型. 当 `Exp` 出现在表示条件判断的位置时 (例如 `if` 和 `while`) 需要为 `bool` 类型. 禁止 `int` 和 `bool` 之间的隐式转换, 这点和 C 或原版 SysY 不同. 
* `ConstExp`: 其中使用的 `IDENT` 必须是常量.

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
integer-const       ::= digit { digit };
```

数值常量的范围为 $[0, 2^{64} - 1]$, 不包含负号.

### 注释

SysY 语言中注释的规范与 C 语言一致, 如下:

* 单行注释: 以序列 `//` 开始, 直到换行符结束, 不包括换行符.
* 多行注释: 以序列 `/*` 开始, 直到第一次出现 `*/` 时结束, 包括结束处 `*/`.

关于其他信息, 请参考 [ISO/IEC 9899](http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1124.pdf) 第 66 页关于注释的定义.

## 语义约束

符合[文法定义](/misc-app-ref/sysy?id=%e6%96%87%e6%b3%95%e5%ae%9a%e4%b9%89)的程序集合是合法的 SysY 语言程序集合的超集. 下面, 我们进一步给出 SysY 语言的语义约束.

### 编译单元

```ebnf
CompUnit ::= [CompUnit] (Decl | FuncDef);
Decl ::= ConstDecl | VarDecl;
```

1. 一个 SysY 程序由单个文件组成, 文件内容对应 EBNF 表示中的 `CompUnit`. 在该 `CompUnit` 中, 必须存在且仅存在一个标识为 `main`, 无参数, 返回类型为 `int` 的 `FuncDef` (函数定义). `main` 函数是程序的入口点.
2. `CompUnit` 的顶层变量/常量声明语句 (对应 `Decl`), 函数定义 (对应 `FuncDef`) 都不可以重复定义同名标识符 (`IDENT`), 即便标识符的类型不同也不允许.
3. `CompUnit` 的变量/常量/函数声明的作用域从该声明处开始, 直到文件结尾.

### 常量定义

```ebnf
ConstDecl     ::= "const" BType ConstDef {"," ConstDef} ";";
ConstDef      ::= IDENT "=" ConstInitVal
                | IDENT "[" ConstExp "]" {"[" ConstExp "]"};
ConstInitVal  ::= ConstExp;
```

1. `ConstDef` 用于定义符号常量. `ConstDef` 中的 `IDENT` 为常量的标识符, 在 `IDENT` 后可以是可选的数组维度和各维长度的定义部分, 也可以是 `=` 之后赋初始值.
2. `ConstDef` 的数组维度和各维长度的定义部分不存在时, 表示定义单个常量. 
3. `ConstDef` 的数组维度和各维长度的定义部分存在时, 表示定义数组. 其语义和 C 语言一致, 比如 `[2][8/2][1*3]` 表示三维数组, 第一到第三维长度分别为 2, 4 和 3, 每维的下界从 0 开始编号. `ConstDef` 中表示各维长度的 `ConstExp` 都必须能在编译时被求值到非负整数. SysY 在声明数组时各维长度都需要显式给出, 而不允许是未知的.
4. 当 `ConstDef` 定义的是数组时, `=` 右边的 `ConstInitVal` 表示常量初值. `ConstInitVal` 中的 `ConstExp` 是能在编译时被求值的 `int` 或者 `bool` 型表达式, 其中可以引用已定义的符号常量.


> SysY 中 “常量” 的定义和 C 语言中的定义有所区别: SysY 中, 所有的常量必须能在编译时被计算出来; 而 C 语言中的常量仅代表这个量不能被修改.
<br><br>
SysY 中的常量有些类似于 C++ 中的 `consteval`, 或 Rust 中的 `const`.

### 变量定义

```ebnf
VarDef ::= IDENT {"[" ConstExp "]"}
         | IDENT {"[" ConstExp "]"} "=" InitVal;
```

1. `VarDef` 用于定义变量. 当不含有 `=` 和初始值时, 其运行时实际初值未定义.
2. `VarDef` 的数组维度和各维长度的定义部分不存在时, 表示定义单个变量; 存在时, 和 `ConstDef` 类似, 表示定义多维数组. (参见 `ConstDef` 的第 2/3 点)
3. 当 `VarDef` 含有 `=` 和初始值时, `=` 右边的 `InitVal` 和 `CostInitVal` 的结构要求相同, 唯一的不同是 `ConstInitVal` 中的表达式是 `ConstExp` 常量表达式, 而 `InitVal` 中的表达式可以是当前上下文合法的任何 `Exp`.
4. `VarDef` 中表示各维长度的 `ConstExp` 必须能被求值到非负整数.

### 函数形参与实参

```ebnf
FuncFParam ::= BType IDENT ["[" "]" {"[" ConstExp "]"}];
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
2. 当 `LVal` 表示数组时, 方括号个数必须和数组变量的维数相同 (即定位到元素). 若 `LVal` 表示的数组作为数组参数参与函数调用, 则数组的方括号个数可以不与维数相同 (参考 [函数形参与实参](/misc-app-ref/sysy-spec?id=函数形参与实参)).
3. 当 `LVal` 表示单个变量时, 不能出现后面的方括号.

### 表达式

```ebnf
Exp ::= LOrExp;
...
```

1. `Exp`: SysY 中表达式的类型为 `int` 或 `bool` 型. 当 `Exp` 出现在表示条件判断的位置时 (例如 `if` 和 `while`) 需要为 `bool` 类型. 
2. 禁止 `int` 和 `bool` 之间的隐式转换, 这点和 C 或原版 SysY 不同.
3. 对于 `||` 和 `&&`, 其左右操作数必须为 `bool` 类型. 上述两种表达式**不满足** C 语言中的短路求值规则.
4. 对于 `!` 运算符, 其操作数必须为 `bool` 类型, 其余操作符的操作数必须为 `int` 类型.
3. `LVal` 必须是当前作用域内, 该 `Exp` 语句之前曾定义过的变量或常量. 赋值号左边的 `LVal` 必须是变量.
4. 函数调用形式是 `IDENT "(" FuncRParams ")"`, 其中的 `FuncRParams` 表示实际参数. 实际参数的类型和个数必须与 `IDENT` 对应的函数定义的形参完全匹配.
5. SysY 中算符的优先级与结合性与 C 语言一致, 上一节定义的 SysY 文法中已体现了优先级与结合性的定义.

> 这种禁止不同类型间隐式转换的特性一般被称为**强类型**. 这里规定 `if` 和 `while` 的条件表达式类型必须为 `bool` 类型并禁止 `int` 和 `bool` 类型的隐式转换的设计和 `rust` 相同, `rust` 就是一种强类型的语言. 