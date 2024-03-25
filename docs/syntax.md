# 实验1 词法与语法分析

!!! warning 注意 
    请及时关注工具/模板/样例的更新, 这些材料都会放在[这里](https://git.zju.edu.cn/accsys)

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

> 其实是简化了两次的 SysY, 我们甚至删去了 Const 和数组初始化列表这两个颇为麻烦的语法.

你的编译器必须支持一个命令行参数的情形. 假设你的程序名为 compiler 且在当前目录下, 那么你的编译器应该能够以以下方式运行：

```bash
./compiler <input_file>
```

该程序必须接受一个输入的源代码文件名作为参数, 我们只会使用这种方式来测试你的编译器. 

<!-- 我们并不会对你打印的语法树格式进行测试, 在 `tests/lab1` 目录下有我们参考编译器生成的语法树, 你可以参考它们的格式调试你的编译器.  -->

### 测试

[测试样例](https://git.zju.edu.cn/accsys/accipit/-/tree/main/tests)放在了 `tests` 文件夹下。我们提供了一个测试脚本 `test.py` 来测试你的编译器, 使用 `python test.py -h` 查看帮助. 

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

???tip "关于 Parser"
    
    由于我们的测试只针对输入输出, 你完全可以不使用 Flex 和 Bison 而去使用 ANTLR 等工具. Bison/Yacc/ANTLR 这类工具一般被称为 Parser Generator, 他们接受一系列文法定义然后生成一个 parser (更确切的说 Bison 生成的是 parser 的源码, 这样甚至不需要有任何额外的依赖). 

    你也可以自己尝试写一个词法分析器和基于递归下降的语法分析器. 这件事并不像你想象的那样困难, 我们推荐 [crafting interpreter](https://craftinginterpreters.com/contents.html) 中语法分析内容供你参考. 

    如果你想尝试手写递归下降或者想尝试 Parser Generator 之外的语法分析手段, 助教推荐一种叫 Parser Combinator 的范式. 一般 Parser Combinator 库会提供一些简单的 parser (比如说"吃掉一个固定的字符串", 你在手写递归下降的时候也会用到) 并提供组合他们的方法 (也就是 combinator, 比如说串联/选择). 所以本质上 Parser Combinator 是一种编写递归下降 parser 的工具.

    也许课上讲的顺序是从递归下降分析过渡到 LL(k) 预测分析的, 你可能会觉得递归下降 parser 识别的语言是 LL(k) 识别的语言的子集, 但是实际上允许回溯的递归下降 parser 是更强的. 我们用 [Parsing Expression Grammar](https://en.wikipedia.org/wiki/Parsing_expression_grammar) (PEG) 来形式化描述递归下降 parser, 正如 CFG 可以对应 Pushdown Automaton.

    PEG 的神奇之处在于, 它可以识别一些非 CFG 语言, 比如 $a^n b^n c^n$ (也许你在计算理论里见过如何通过 pumping lemma 证明这个语言不是 CFL). 但这并不意味着 PEG 比 CFG 强, 首先你在实际使用的时候仍然需要考虑左递归的问题, 其次有一些 CFL 难以用 PEG 识别, 比如说回文串. 目前是否存在一个无法被 PEG 识别的 CFL 仍然是一个 open problem. 

    你可以在[这个项目](https://git.zju.edu.cn/accsys/peg-test)里看到一个 PEG parser 是如何识别 $a^n b^n c^n$ 的, 以及为什么它难以识别回文串(项目里的写法实际上只能识别一个长度为 3 的回文前缀, 这和 PEG 的确定性有关).

    我们的 [IR 工具](https://git.zju.edu.cn/accsys/accipit)和[样例编译器](https://git.zju.edu.cn/accsys/accsys-rs)使用了 Rust 的 Parser Combinator 库 [chumsky](https://github.com/zesterer/chumsky) 和 [nom](https://github.com/rust-bakery/nom), 他们都支持回溯功能, 所以可以识别任何 PEG, 也即可以生成(带回溯的)递归下降 parser.

## 实验提交

实验一和实验二统一提交一次. 你需要提供:

1. 源程序的压缩包. 
2. 一份 PDF 格式的实验报告, 内容包括:

    - 你的程序实现了哪些功能? 简要说明如何实现这些功能.
    - 你的程序应该如何被编译? 请详细说明应该如何编译你的程序. 无法顺利编译将导致助教无法对你的程序所实现的功能进行任何测试, 从而丢失相应的分数.
    - 实验报告的长度不得超过 6 页. 所以实验报告中需要重点描述的是你的程序中的亮点, 是你认为最个性化/最具独创性的内容, 尤其要避免大段地向报告里贴代码.

> 建议使用 Git 管理你的代码

## 实现建议

!!!tip "AST表示与打印"
    AST 是编译器的核心数据结构之一, 在尝试使用不同编程语言时会有不同的技术方案, 我们以 `Stmt` 为例, 介绍在 C, C++, Rust 中表示语法树这类数据结构的一种方法. 

    === "C"

        C语言常用的技巧是 enum + union:
        ``` c title="ast.h"
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
        而在打印语法树时, 只需要 switch 递归遍历即可:

        ```c title="ast.h"
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
                default:
                    raise_error();
            }
        }
        ```

    === "C++"
        C++ 除了可以使用 C 的方法外, 还可以使用基于面向对象的技术：

        ``` c++ title="ast.h"
        struct Node {
        };

        struct IfStmt : public Node {
            Node *condition;
            Node *then;
            Node *els;
            IfStmt(Node *condition, Node *then, Node *els) {}
        }
        ```

        在打印语法树时, 基于面向对象的技术可以使用虚函数来实现: 

        ``` c++ title="ast.h"
        struct Node {
            virtual void print() = 0;
        };
        
        struct IfStmt : public Node {
            Node *condition;
            Node *then;
            Node *els;
            IfStmt(Node *condition, Node *then, Node *els) {/* ... */}
            void print() override {
                printf("If\n");
                condition->print();
                printf("Then\n");
                then->print();
                if (els != nullptr) {
                    printf("Else\n");
                    els->print();
                }
            }
        }
        ...
        ```

        不过你可以考虑自己实现一个简易的 RTTI (Runtime Type Identification), 类似于 python 中的 `isinstance`, 本质上还是 C 的 enum + union.

        ```c++ title="ast.h"
        enum NodeKind {
            ND_Expr,
            ND_IfStmt,
            ND_WhileStmt,
            ...
        }
        struct Node {
            NodeKind node_kind;
            Node(NodeKind kind): node_kind(kind) { }
            template <typename T>
            bool is() {
                return value_type == std::remove_pointer_t<T>::this_kind;
            }
            template <typename T>
            T as() {
                if (is<T>()) {
                    return static_cast<T>(this);
                } else {
                    return nullptr;
                }
            }
        };
        
        struct IfStmt : public Node {
            constexpr static NodeKind this_kind = ND_IfStmt;
            Node *condition;
            Node *then;
            Node *els;
            IfStmt(Node *condition, Node *then, Node *els): Node(this_kind) {/* ... */}
        }

        void print(Node *node) {
            if (IfStmt *stmt = node->as<IfStmt *>()) {
                /* ... */
            }
        }
        ```

    === "Rust/OCaml/Modern C++"
        熟悉 Rust 的同学一定会使用 enum, 即枚举类型来表示AST：

        ```rust
        enum Node {
            Expr(Expr),
            IfStmt(IfStmt),
            WhileStmt(WhileStmt),
            ReturnStmt(ReturnStmt),
        }

        struct IfStmt {
            ...
        }
        ```

        在打印语法树时, 我们可以使用 match 语法来实现：

        ```rust
        fn print_node(node: &Node) {
            match node {
                Node::Expr(expr) => print_expr(expr),
                Node::IfStmt(if_stmt) => print_if_stmt(if_stmt),
                Node::WhileStmt(while_stmt) => print_while_stmt(while_stmt),
                Node::ReturnStmt(return_stmt) => print_return_stmt(return_stmt),
            }
        }
        ```

        这套东西叫代数数据类型 (Algebraic Data Type, ADT) 和模式匹配 (pattern matching), 你可以在很多函数式编程语言中见到它 (比如 OCaml/Haskell). 
        
        当然如果你足够热爱 C++ 或者足够痛恨 Rust 还想用这套的话也不是不行, 这里给出一个基于 `std::variant` 和 `std::visit` 的[实现](https://gcc.godbolt.org/z/jKMacTW37) (这个方法来自于 [cppreference](https://en.cppreference.com/w/cpp/utility/variant/visit)). 这就是 Modern C++.

我们给出了一个使用 rust 编写的样例 [parser](https://git.zju.edu.cn/accsys/accsys-rs), 它足以通过 Lab 1 的测试. 同时我们提供了一份 CMake 的[项目模板](https://git.zju.edu.cn/accsys/accsys-cmake-template), 里面引入了 `fmt` 库, 并以上述实现建议中 C++ 的第二种写法实现了一个简单的表达式 parser (这种风格更接近 LLVM, 你会在 Lab 3 中再次见到它).

当然我们不要求你必须使用这个模板, 你可以自行编写构建系统 (make/xmake/cargo/dune) 但要在报告中注明构建方法. 上面所说的所有工具和风格仅作推荐, 你不必受此约束. 
