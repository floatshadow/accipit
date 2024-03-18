# 实验1 词法与语法分析

## 词法分析

词法分析的目的是将源代码(一个个字符)解析成一个个token. Token 就是词法分析的最小单元, 表示一个程序有意义的最小分割. 下表为常见的token类型和样例:

| 类型  | 样例                       |
|-----|--------------------------|
| 标识符 | x, sum, i                |
| 关键字 | if, while, return        |
| 分隔符 | }, (, ), ;               |
| 运算符 | +, < , =                 |
| 字面量 | true, 666, "hello world" |
| 注释  | // this is a comment.    |

```c
x = a + b * 2;
```

上面的简单代码会被解析为:
`[(identifier, x), (operator, =), (identifier, a), (operator, +), (identifier, b), (operator, *), (literal, 2), (separator, ;)]`.

词法分析并不复杂, 本质上就是一个有限状态机([正则文法](https://en.wikipedia.org/wiki/Regular_expression))的匹配. 只需要简单的遍历源代码中的字符, 根据字符的值分别判断即可.

## 语法分析

由词法分析得到的一个个token在语法分析阶段进行进一步的解析. 具体来说, 需要:

1. 对于一串合法的tokens, 生成语法树.
2. 对于的一串**不合法**的token, 检测到可能的错误并报告给用户.

如果说词法分析由[正则文法](https://en.wikipedia.org/wiki/Regular_expression)为基础, 语法分析则以[上下文无关文法](https://en.wikipedia.org/wiki/Context-free_grammar)作为基石. 

而在实际开发过程中, 我们往往会选用一些现有的工具来自动生成词法分析器和语法分析器. 本实验中我们建议大家使用 Flex, Bison 工具链来进行词法与语法分析. 

## 你的任务

你需要完成编译器的词法分析和语法分析部分, 能够解析出符合词法与语法的 SysY 语言源代码, 并且我们**强烈建议**你实现一个打印语法树的函数, 以便于后续调试. 以如下的阶乘函数为例
```c
int factorial(int n) {
    if (n == 0)
        return 1;
    return n * factorial(n - 1);
}
int main() {
    int n = getint();
    int result = factorial(n);
    putint(result);
    return 0;
}
```
你打印出的语法树可以是这样的：

```text
 CompUnit
 ├─ FuncDef factorial 'int(int)'
 │  ├─ FuncFParam n 'int'
 │  └─ Block
 │     ├─ IfStmt
 │     │  ├─ RelationOp ==
 │     │  │  ├─ Ident n
 │     │  │  └─ IntConst 0
 │     │  └─ ReturnStmt
 │     │     └─ IntConst 1
 │     └─ ReturnStmt
 │        └─ BinaryOp *
 │           ├─ Ident n
 │           └─ Call factorial
 │              └─ BinaryOp -
 │                 ├─ Ident n
 │                 └─ IntConst 1
 └─ FuncDef main 'int()'
    └─ Block
       ├─ VarDecl
       │  └─ Ident n
       │     └─ Call getint
       ├─ VarDecl
       │  └─ Ident result
       │     └─ Call factorial
       │        └─ Ident n
       ├─ ExpStmt
       │  └─ Call putint
       │     └─ Ident result
       └─ ReturnStmt
          └─ IntConst 0
```

为了减轻你的负担, 我们只要求你实现**简化版**的 SysY 语言, 见[附录](appendix/sysy-spec.md). 

你的编译器必须支持一个命令行参数的情形. 假设你的程序名为 compiler 且在当前目录下, 那么你的编译器应该能够以以下方式运行：

```bash
./compiler <input_file>
```

该程序必须接受一个输入的源代码文件名作为参数, 我们只会使用这种方式来测试你的编译器. 

<!-- 我们并不会对你打印的语法树格式进行测试, 在 `tests/lab1` 目录下有我们参考编译器生成的语法树, 你可以参考它们的格式调试你的编译器.  -->

### 测试

我们提供了一个测试脚本 `test.py` 来测试你的编译器, 使用 `python test.py -h` 查看帮助. 

假设你的程序名为 compiler 且在当前目录下, 运行以下命令来测试你的编译器：

```bash
python3 test.py ./compiler lab1
```

在 lab1 中, 我们提供了以下两种测试样例：

- 如 `tests/lab1/main.sy` 所示, 源文件中的语法是正确的, 你的编译器应该能够正确的解析出语法树并退出. 

```c
int main(){
    return 0;
}
```

- 如 `tests/lab1/error2.sy` 所示, 源文件中的语法是错误的, 你的编译器应该能够检测出错误并报错后以非0返回值退出. 

```c
// Syntax Error Line 8: identifier "3c"

int foo(int x){
  return x + 1;
}

int main(){
  int 3c = foo(1);
}
```

我们的测试完全采用输入输出形式, 即对于符合语法的源代码, 你的程序正常运行后正常退出, 对于不符合语法的源代码, 你的程序应该能够检测出错误报错后返回非0值, 我们对报错的格式没有要求. 测试文件中的注释供你参考. 

!!!tip "关于 Parser"
    由于我们的测试只针对输入输出, 你完全可以不使用 Flex 和 Bison. Bison/Yacc 这类工具一般被称为 Parser Generator, 他们接受一系列文法定义然后生成一个 Parser (更确切的说 Bison 生成的是 Parser 的源码, 这样甚至不需要有任何额外的依赖). 

    你也可以自己尝试写一个词法分析器和基于递归下降的语法分析器. 这件事并不像你想象的那样困难, 我们推荐 [crafting interpreter](https://craftinginterpreters.com/contents.html) 中语法分析内容供你参考. 

    如果你想尝试手写递归下降或者想尝试 Parser Generator 之外的语法分析手段, 助教推荐一种叫 Parser Combinator 的范式.

## 实验提交

1. 源程序的压缩包, 即 `src` 目录下的所有文件. 
2. 一份PDF格式的实验报告, 内容包括:
    - 你的程序实现了哪些功能? 简要说明如何实现这些功能
    - 你的程序应该如何被编译? 请详细说明应该如何编译你的程序. 无法顺利编译将导致助教无法对你的程序所实现的功能进行任何测试, 从而丢失相应的分数
    - 实验报告的长度不得超过3页. 所以实验报告中需要重点描述的是你的程序中的亮点, 是你认为最个性化、最具独创性的内容, 尤其要避免大段地向报告里贴代码

> 记得在完成实验时使用 Git 在本地提交你的代码. 

## 实现建议

!!!tip "AST表示与打印"
    AST 是编译器的核心数据结构之一, 在尝试使用不同编程语言时会有不同的技术方案,  我们以 `Stmt` 为例, 介绍在 C, C++, Rust 中表示语法树这类数据结构的一种方法. 

    === "C"

        C语言常用的技巧是 enum + union：
        ``` c
        enum StmtKind {
            STMT_EXPR,
            STMT_IF,
            STMT_WHILE,
            STMT_RETURN,
        };

        struct Stmt {
            enum StmtKind kind;
            union {
                struct Expr *expr;
                struct IfStmt *if_stmt;
                struct WhileStmt *while_stmt;
                struct ReturnStmt *return_stmt;
            };
        }; 
        ```

        我们使用一个枚举类型来表示语法树的节点类型, 并且使用 union 来存储不同类型的节点. 
        而在打印语法树时, 只需要 switch 递归遍历即可：

        ``` c
        void print_stmt(struct Stmt *stmt) {
            switch (stmt->kind) {
                case STMT_EXPR:
                    print_expr(stmt->expr);
                    break;
                case STMT_IF:
                    print_if_stmt(stmt->if_stmt);
                    break;
                case STMT_WHILE:
                    print_while_stmt(stmt->while_stmt);
                    break;
                case STMT_RETURN:
                    print_return_stmt(stmt->return_stmt);
                    break;
            }
        }
        ```

    === "C++"

        C++ 除了可以使用C的方法外, 还可以使用基于面向对象的技术：

        ``` c++
        class BaseStmt {
        };

        class ExprStmt : public BaseStmt {
        public:
            ExprStmt(Expr *expr) : expr_(expr) {}
        private:
            Expr *expr_;
        };

        class IfStmt : public BaseStmt {
            ...
        }
        ```

        在打印语法树时, 基于面向对象的技术可以使用虚函数来实现：

        ``` c++
        class BaseStmt {
        public:
            virtual void print() = 0;
        };
        
        class ExprStmt : public BaseStmt {
        public:
            ExprStmt(Expr *expr) : expr_(expr) {}
            void print() override {
                expr_->print();
            }
        private:
            Expr *expr_;
        };
        
        ...
        ```

    === "Rust"
        熟悉Rust的同学一定会使用 enum, 即枚举类型来表示AST：

        ```rust
        enum Stmt {
            ExprStmt(Expr),
            IfStmt(IfStmt),
            WhileStmt(WhileStmt),
            ReturnStmt(ReturnStmt),
        }  

        struct ExprStmt {
            expr: Expr,
        }

        struct IfStmt {
            ...
        }
        ```

        在打印语法树时, 我们可以使用 match 语法来实现：

        ```rust
        fn print_stmt(stmt: &Stmt) {
            match stmt {
                Stmt::ExprStmt(expr_stmt) => print_expr(&expr_stmt.expr),
                Stmt::IfStmt(if_stmt) => print_if_stmt(if_stmt),
                Stmt::WhileStmt(while_stmt) => print_while_stmt(while_stmt),
                Stmt::ReturnStmt(return_stmt) => print_return_stmt(return_stmt),
            }
        }
        ```

该实验技术难度并不大, 但需要同学们掌握 Flex, Bison 工具链的使用并且实现整个语言的词法分析和语法分析, 整体过程极为繁琐, 我们建议同学们尽早开始实验. 